<script lang="ts">
  import type { ArtifactDigest } from "@bindings/cob/release/ArtifactDigest";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";
  import type { RepoRefs } from "@bindings/repo/RepoRefs";
  import type { Tag } from "@bindings/repo/Tag";

  import { invoke } from "@app/lib/invoke";
  import { disableHide, enableHide, forceHide } from "@app/lib/modal";
  import * as router from "@app/lib/router";
  import { basename, formatBytes, formatOid, isCommit } from "@app/lib/utils";

  import Button from "@app/components/Button.svelte";
  import Checkbox from "@app/components/Checkbox.svelte";
  import DropdownList from "@app/components/DropdownList.svelte";
  import DropdownListItem from "@app/components/DropdownListItem.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Popover, { closeFocused } from "@app/components/Popover.svelte";
  import RepoAvatar from "@app/components/RepoAvatar.svelte";
  import ScrollArea from "@app/components/ScrollArea.svelte";
  import TextInput from "@app/components/TextInput.svelte";

  interface Props {
    repo: RepoInfo;
  }

  const { repo }: Props = $props();

  interface NamedTag {
    name: string;
    tag: Tag;
  }

  interface StagedArtifact {
    path: string;
    name: string;
    hashing: boolean;
    digest?: ArtifactDigest;
    error?: string;
  }

  let tags: NamedTag[] = $state([]);
  let selectedTag: NamedTag | undefined = $state();
  let commitInput = $state("");
  let staged: StagedArtifact[] = $state([]);
  let submitting = $state(false);
  let submitError: string | undefined = $state();
  let seedFailures = $state(0);
  let popoverExpanded = $state(false);
  let nodeRunning = $state(false);
  let seedAfterPublish = $state(true);
  // Seeding is what makes the bytes reachable; registering alone only records
  // the content ids. It needs the artifact node, so the option goes away when
  // the node is down rather than failing at publish time.
  const canSeed = $derived(nodeRunning && seedAfterPublish);

  // A release is keyed by a commit. An annotated tag additionally gives it a
  // name and title, so prefer one when the user picked a tag; the store
  // rejects lightweight tags, whose `tagOid` is absent.
  const target = $derived.by(() => {
    if (selectedTag) {
      return {
        oid: selectedTag.tag.oid,
        tag: selectedTag.tag.tagOid,
      };
    }
    const oid = commitInput.trim();
    return isCommit(oid) ? { oid, tag: undefined } : undefined;
  });

  const ready = $derived(
    staged.length > 0 &&
      staged.every(a => a.digest !== undefined) &&
      target !== undefined,
  );

  $effect(() => {
    if (staged.length > 0 || selectedTag || commitInput.trim() !== "") {
      disableHide();
    } else {
      enableHide();
    }
  });

  // Tags live per remote; collapse them into one list by name, preferring an
  // annotated entry so a release can carry the tag's title.
  $effect(() => {
    void invoke<boolean>("artifact_node_running")
      .then(running => {
        nodeRunning = running;
      })
      .catch(() => {
        nodeRunning = false;
      });
  });

  $effect(() => {
    void invoke<RepoRefs>("list_repo_refs", { rid: repo.rid }).then(refs => {
      const byName: Record<string, Tag> = {};
      const add = (entries: Record<string, Tag>) => {
        for (const [name, tag] of Object.entries(entries)) {
          const existing = byName[name];
          if (!existing || (!existing.tagOid && tag.tagOid)) {
            byName[name] = tag;
          }
        }
      };
      add(refs.canonical.tags);
      for (const remote of refs.remotes) {
        add(remote.tags);
      }
      tags = Object.entries(byName)
        .map(([name, tag]) => ({ name, tag }))
        .sort((a, b) => b.tag.timestamp - a.tag.timestamp);
    });
  });

  async function stage(paths: string[]) {
    const added = paths
      .filter(path => !staged.some(a => a.path === path))
      .map(path => ({ path, name: basename(path), hashing: true }));
    if (added.length === 0) {
      return;
    }
    staged = [...staged, ...added];

    await Promise.all(
      added.map(async entry => {
        try {
          const digest = await invoke<ArtifactDigest>("compute_artifact_cid", {
            path: entry.path,
          });
          staged = staged.map(a =>
            a.path === entry.path ? { ...a, digest, hashing: false } : a,
          );
        } catch {
          staged = staged.map(a =>
            a.path === entry.path
              ? { ...a, hashing: false, error: "Could not read file" }
              : a,
          );
        }
      }),
    );
  }

  async function addFiles() {
    await stage(await invoke<string[]>("pick_artifact_files"));
  }

  async function addDirectory() {
    const path = await invoke<string | null>("pick_artifact_directory");
    if (path) {
      await stage([path]);
    }
  }

  async function submit() {
    if (!target || !ready) {
      return;
    }
    submitting = true;
    submitError = undefined;
    seedFailures = 0;
    try {
      const releaseId = await invoke<string>("create_or_open_release", {
        rid: repo.rid,
        oid: target.oid,
        tag: target.tag,
      });
      // Registered one at a time: each is its own signed COB entry, and a
      // failure part-way leaves the earlier ones intact.
      for (const artifact of staged) {
        if (!artifact.digest) {
          continue;
        }
        await invoke("register_artifact", {
          rid: repo.rid,
          releaseId,
          cid: artifact.digest.cid,
          name: artifact.name,
          sizeBytes: artifact.digest.sizeBytes,
        });

        if (canSeed) {
          // A seed failure leaves the artifact registered but unreachable,
          // which the release page can recover from later, so it must not
          // discard the release we just published.
          try {
            await invoke("seed_artifact", {
              rid: repo.rid,
              releaseId,
              cid: artifact.digest.cid,
              sourcePath: artifact.path,
            });
          } catch {
            seedFailures += 1;
          }
        }
      }
      // Both creating and registering are idempotent, so when seeding fails
      // the modal stays open and a retry re-seeds rather than duplicating the
      // release.
      if (seedFailures > 0) {
        return;
      }

      enableHide();
      await router.push({
        resource: "repo.release",
        rid: repo.rid,
        release: releaseId,
        allAuthors: false,
      });
      forceHide();
    } catch {
      submitError = "Could not create the release.";
    } finally {
      submitting = false;
    }
  }
</script>

<style>
  .modal {
    width: 44rem;
    font: var(--txt-body-m-regular);
    display: flex;
    flex-direction: column;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-lg);
    background-color: var(--color-surface-canvas);
    overflow: hidden;
  }
  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 1.5rem;
    height: 3.25rem;
    flex-shrink: 0;
    border-bottom: 1px solid var(--color-border-subtle);
  }
  .header-left {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
    min-width: 0;
  }
  .repo-name {
    font: var(--txt-body-m-semibold);
    color: var(--color-text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .title {
    font: var(--txt-body-m-regular);
    color: var(--color-text-primary);
    white-space: nowrap;
  }
  .body {
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }
  .section {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .section-title {
    font: var(--txt-body-m-semibold);
    color: var(--color-text-secondary);
  }
  .hint {
    font: var(--txt-body-m-regular);
    color: var(--color-text-tertiary);
  }
  .target-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  .artifact {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--color-border-subtle);
    font: var(--txt-body-m-regular);
  }
  .artifact:last-child {
    border-bottom: none;
  }
  .artifact-name {
    flex: 1;
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .artifact-meta {
    font: var(--txt-body-m-regular);
    color: var(--color-text-tertiary);
    white-space: nowrap;
  }
  .artifact-list {
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
  }
  .error {
    font: var(--txt-body-m-regular);
    color: var(--color-foreground-red);
  }
  .footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    padding: 1rem 1.5rem;
    border-top: 1px solid var(--color-border-subtle);
  }
  .actions {
    display: flex;
    gap: 0.5rem;
    margin-left: auto;
  }
</style>

<div class="modal">
  <div class="header">
    <div class="header-left">
      <RepoAvatar
        name={repo.payloads["xyz.radicle.project"]?.data.name ?? ""}
        rid={repo.rid}
        styleWidth="1rem" />
      <span class="repo-name">
        {repo.payloads["xyz.radicle.project"]?.data.name}
      </span>
      <Icon name="chevron-right" />
      <span class="title">New release</span>
    </div>
    <Button variant="naked" onclick={forceHide}>
      <span style:color="var(--color-text-tertiary)">
        <Icon name="close" />
      </span>
    </Button>
  </div>

  <div class="body">
    <div class="section">
      <span class="section-title">Target</span>
      <div class="target-row">
        <Popover
          popoverPadding="0"
          placement="bottom-start"
          styleZIndex="400"
          bind:expanded={popoverExpanded}>
          {#snippet toggle(onclick)}
            <Button
              variant="outline"
              {onclick}
              disabled={tags.length === 0}
              active={popoverExpanded}
              title={tags.length === 0 ? "This repo has no tags" : undefined}>
              <Icon name="label" />
              <span style:color="var(--color-text-secondary)">
                {selectedTag ? selectedTag.name : "Select a tag"}
              </span>
              <Icon name={popoverExpanded ? "chevron-up" : "chevron-down"} />
            </Button>
          {/snippet}
          {#snippet popover()}
            <div
              style:border="1px solid var(--color-border-subtle)"
              style:border-radius="var(--border-radius-sm)"
              style:background-color="var(--color-surface-canvas)">
              <DropdownList items={tags} styleDropdownMinWidth="16rem">
                {#snippet item(entry)}
                  <DropdownListItem
                    selected={selectedTag?.name === entry.name}
                    styleGap="0.5rem"
                    onclick={() => {
                      selectedTag = entry;
                      commitInput = "";
                      closeFocused();
                    }}>
                    <Icon name="label" />
                    <span style:color="var(--color-text-secondary)">
                      {entry.name}
                    </span>
                    <span class="artifact-meta">
                      {formatOid(entry.tag.oid)}
                    </span>
                  </DropdownListItem>
                {/snippet}
                {#snippet empty()}
                  <div class="hint" style:padding="0.5rem">No tags</div>
                {/snippet}
              </DropdownList>
            </div>
          {/snippet}
        </Popover>

        <span class="hint">or</span>

        <div style:flex="1">
          <TextInput
            placeholder="Commit SHA"
            bind:value={commitInput}
            oninput={() => {
              if (commitInput.trim() !== "") {
                selectedTag = undefined;
              }
            }} />
        </div>
      </div>
      {#if selectedTag && !selectedTag.tag.tagOid}
        <span class="hint">
          Lightweight tag: the release records the commit only.
        </span>
      {/if}
    </div>

    <div class="section">
      <span class="section-title">Artifacts</span>
      {#if staged.length > 0}
        <ScrollArea style="max-height: 16rem;">
          <div class="artifact-list">
            {#each staged as artifact (artifact.path)}
              <div class="artifact">
                <Icon name={artifact.error ? "warning" : "attach"} />
                <span class="artifact-name" title={artifact.path}>
                  {artifact.name}
                </span>
                {#if artifact.hashing}
                  <span class="artifact-meta">Hashing…</span>
                {:else if artifact.error}
                  <span class="error">{artifact.error}</span>
                {:else if artifact.digest}
                  <span class="artifact-meta">
                    {formatBytes(artifact.digest.sizeBytes)}
                  </span>
                {/if}
                <Button
                  variant="naked"
                  onclick={() => {
                    staged = staged.filter(a => a.path !== artifact.path);
                  }}>
                  <span style:color="var(--color-text-tertiary)">
                    <Icon name="close" />
                  </span>
                </Button>
              </div>
            {/each}
          </div>
        </ScrollArea>
      {:else}
        <span class="hint">
          Attach the files this release publishes. They are content-addressed
          locally; seeding them is a separate step.
        </span>
      {/if}
      <div class="target-row">
        <Button variant="outline" onclick={addFiles}>
          <Icon name="plus" />Add files
        </Button>
        <Button variant="outline" onclick={addDirectory}>
          <Icon name="plus" />Add directory
        </Button>
      </div>

      <div
        style:margin-top="0.25rem"
        style:width="fit-content"
        title={nodeRunning
          ? undefined
          : "The artifact node is not running, so the release will record the content ids only."}>
        <Checkbox bind:checked={seedAfterPublish} disabled={!nodeRunning}>
          Seed these artifacts so others can download them
        </Checkbox>
      </div>
    </div>
  </div>

  <div class="footer">
    {#if submitError}
      <span class="error">{submitError}</span>
    {:else if seedFailures > 0}
      <span class="error">
        Published, but {seedFailures}
        {seedFailures === 1 ? "artifact" : "artifacts"} could not be seeded.
      </span>
    {/if}
    <div class="actions">
      <Button variant="outline" onclick={forceHide}>Cancel</Button>
      <Button
        variant="secondary"
        disabled={!ready || submitting}
        onclick={submit}>
        {submitting ? "Creating…" : "Create release"}
      </Button>
    </div>
  </div>
</div>
