<script lang="ts">
  // Modal-like create flow used inline on the Releases list view.
  // Lifecycle:
  //   1. user enters a commit/tag OID
  //   2. user picks files (or drops them onto the window)
  //   3. for each file, compute CID locally so the user sees what
  //      they're about to publish
  //   4. submit: create_or_open_release, then register_artifact per file,
  //      and seed_artifact if the auto-seed setting is on

  import type { PaginatedQuery } from "@bindings/cob/PaginatedQuery";
  import type { Commit } from "@bindings/repo/Commit";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";
  import type { Tag } from "@bindings/repo/Tag";

  import { get } from "svelte/store";
  import { onMount } from "svelte";

  import { autoSeedArtifacts } from "@app/lib/autoSeed";
  import { invoke } from "@app/lib/invoke";
  import * as router from "@app/lib/router";

  import CommitPicker from "@app/components/CommitPicker.svelte";

  interface Props {
    repo: RepoInfo;
    onCancel: () => void;
  }

  const { repo, onCancel }: Props = $props();

  type StagedFile = {
    path: string;
    name: string;
    cid?: string;
    error?: string;
    computing: boolean;
  };

  let oid = $state("");
  // Selected tag refname. "" means "no tag" — the release is keyed by the
  // commit OID alone. Annotated tags also contribute their tag OID to the
  // COB; lightweight tags only set the commit OID.
  let selectedTagName = $state("");
  const files = $state<StagedFile[]>([]);
  let submitting = $state(false);
  // Seed over iroh after publishing; defaults to the persisted preference
  // but can be toggled per release via the checkbox below.
  let autoSeed = $state(get(autoSeedArtifacts));
  let submitError: string | undefined = $state();
  // Recent commits offered as autocomplete suggestions for the OID input.
  let commits = $state<Commit[]>([]);
  // Tags listed for the repo, surfaced as the primary picker.
  let tags = $state<Tag[]>([]);

  // The annotated-tag OID we'll record alongside the commit, if a tag is
  // selected and it's annotated. Lightweight tags resolve to a commit OID
  // only and don't contribute a separate tag OID.
  const selectedTag = $derived(tags.find(t => t.name === selectedTagName));
  const tagOid = $derived(selectedTag?.annotated ? selectedTag.oid : undefined);

  onMount(async () => {
    try {
      const [commitsResult, tagsResult] = await Promise.all([
        invoke<PaginatedQuery<Commit[]>>("list_repo_commits", {
          rid: repo.rid,
          skip: 0,
          take: 50,
        }),
        invoke<Tag[]>("list_tags", { rid: repo.rid }),
      ]);
      commits = commitsResult.content;
      tags = tagsResult;
    } catch (err) {
      console.error("loading commits/tags failed", err);
    }
  });

  function onSelectTag(name: string) {
    selectedTagName = name;
    if (name === "") return;
    const tag = tags.find(t => t.name === name);
    if (tag) {
      // Auto-fill the commit OID from the tag's target. The user can still
      // override it manually — and doing so clears the tag selection so we
      // don't end up writing a tag OID that doesn't match the commit.
      oid = tag.commit;
    }
  }

  function onCommitInput(value: string) {
    oid = value;
    // If the typed value no longer matches the selected tag's commit, the
    // tag and commit are out of sync — clear the tag.
    if (selectedTag && selectedTag.commit !== value) {
      selectedTagName = "";
    }
  }

  async function pickFiles() {
    const picked = await invoke<string[]>("pick_artifact_files");
    for (const p of picked) {
      await stageFile(p);
    }
  }

  async function pickDirectory() {
    const dir = await invoke<string | null>("pick_artifact_directory");
    if (dir) await stageFile(dir);
  }

  async function stageFile(path: string) {
    const baseName = path.split(/[\\/]/).filter(Boolean).pop() ?? path;
    const idx = files.length;
    files.push({
      path,
      name: baseName,
      computing: true,
    });
    // After push, files[idx] returns Svelte 5's reactive proxy view of the
    // entry — write through that handle, not a local reference, otherwise
    // mutations don't trigger re-renders and the row spins forever.
    try {
      files[idx].cid = await invoke<string>("compute_artifact_cid", { path });
    } catch (err) {
      files[idx].error = String(err);
    } finally {
      files[idx].computing = false;
    }
  }

  function remove(index: number) {
    files.splice(index, 1);
  }

  async function submit() {
    if (!oid.trim()) {
      submitError = "Commit OID is required.";
      return;
    }
    if (files.length === 0) {
      submitError = "Add at least one artifact.";
      return;
    }
    // Every file must have a computed CID before publishing, otherwise it
    // would be silently dropped from the release below.
    if (files.some(f => !f.cid)) {
      submitError = files.some(f => f.computing)
        ? "Wait for all files to finish processing."
        : "Some files failed to process; remove them before publishing.";
      return;
    }
    submitError = undefined;
    submitting = true;
    try {
      const releaseId = await invoke<string>("create_or_open_release", {
        rid: repo.rid,
        oid: oid.trim(),
        tag: tagOid,
      });
      for (const f of files) {
        await invoke("register_artifact", {
          rid: repo.rid,
          releaseId,
          cid: f.cid,
          name: f.name,
        });
        if (autoSeed) {
          try {
            await invoke("seed_artifact", {
              rid: repo.rid,
              releaseId,
              cid: f.cid,
              sourcePath: f.path,
            });
          } catch (err) {
            // Don't block release creation if a single seed fails — the
            // artifact is still recorded on the COB and the user can
            // retry seeding from the detail view.
            console.error("seed failed for", f.cid, err);
          }
        }
      }
      await router.push({
        resource: "repo.release",
        rid: repo.rid,
        release: releaseId,
      });
    } catch (err) {
      submitError = String(err);
    } finally {
      submitting = false;
    }
  }
</script>

<style>
  .form {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding: 1rem;
    background-color: var(--color-surface-1);
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  label,
  .field-label {
    font: var(--txt-body-s-semibold);
    color: var(--color-text-secondary);
  }
  input[type="text"],
  select {
    padding: 0.4rem 0.5rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    background-color: var(--color-surface-canvas);
    font: var(--txt-body-m-regular);
  }
  .files {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  .file-row {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    padding: 0.4rem 0.5rem;
    background-color: var(--color-surface-canvas);
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
  }
  .file-row .top {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  .file-row .path {
    font: var(--txt-body-s-mono);
    color: var(--color-text-secondary);
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .file-row .cid {
    font: var(--txt-body-s-mono);
    color: var(--color-text-secondary);
    word-break: break-all;
  }
  .file-row input.name {
    flex: 0 0 12rem;
  }
  .actions {
    display: flex;
    gap: 0.5rem;
  }
  button {
    padding: 0.4rem 0.75rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    background-color: var(--color-surface-subtle);
    cursor: pointer;
  }
  button.primary {
    background-color: var(--color-fill-accent);
    color: var(--color-text-inverse);
  }
  .error {
    color: var(--color-fill-error);
    font: var(--txt-body-s-regular);
  }
  .toggle {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
</style>

<div class="form">
  <div class="field">
    <label for="release-tag">Tag</label>
    <select
      id="release-tag"
      value={selectedTagName}
      onchange={e => onSelectTag((e.currentTarget as HTMLSelectElement).value)}
      disabled={submitting}>
      <option value="">No tag — release a bare commit</option>
      {#each tags as tag (tag.name)}
        <option value={tag.name}>
          {tag.name}
          {tag.annotated ? "(annotated)" : ""}
          → {tag.commit.slice(0, 7)}
        </option>
      {/each}
    </select>
  </div>

  <div class="field">
    <span class="field-label">Commit</span>
    <CommitPicker
      {commits}
      value={oid}
      disabled={submitting}
      onSelect={onCommitInput} />
  </div>

  <div class="field">
    <span class="field-label">Artifacts</span>
    <div class="files">
      {#each files as file, i (file.path)}
        <div class="file-row">
          <div class="top">
            <input
              class="name"
              type="text"
              bind:value={file.name}
              disabled={submitting} />
            <span class="path" title={file.path}>{file.path}</span>
            <button onclick={() => remove(i)} disabled={submitting}>×</button>
          </div>
          {#if file.computing}
            <span class="cid">computing…</span>
          {:else if file.error}
            <span class="error">{file.error}</span>
          {:else if file.cid}
            <span class="cid">{file.cid}</span>
          {/if}
        </div>
      {/each}
    </div>
    <div class="actions" style:margin-top="0.5rem">
      <button onclick={pickFiles} disabled={submitting}>Add files…</button>
      <button onclick={pickDirectory} disabled={submitting}>
        Add directory…
      </button>
    </div>
  </div>

  <div class="toggle">
    <input
      id="auto-seed"
      type="checkbox"
      bind:checked={autoSeed}
      disabled={submitting} />
    <label for="auto-seed">Seed over iroh after publishing</label>
  </div>

  {#if submitError}
    <div class="error">{submitError}</div>
  {/if}

  <div class="actions" style:justify-content="flex-end">
    <button onclick={onCancel} disabled={submitting}>Cancel</button>
    <button class="primary" onclick={submit} disabled={submitting}>
      {submitting ? "Publishing…" : "Publish release"}
    </button>
  </div>
</div>
