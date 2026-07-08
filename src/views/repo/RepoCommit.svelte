<script lang="ts">
  import type {
    FileNote,
    FileStatus,
  } from "@app/components/diffFileHeaderState.svelte";
  import type { Diff } from "@bindings/diff/Diff";
  import type { FileDiff } from "@bindings/diff/FileDiff";
  import type { Commit } from "@bindings/repo/Commit";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";
  import type { Blob } from "@bindings/source/Blob";
  import type { GitStatusEntry } from "@pierre/trees";

  import { diffOptions } from "@app/lib/diffOptions.svelte";
  import { getDiffText, invoke } from "@app/lib/invoke";
  import * as router from "@app/lib/router";
  import {
    absoluteTimestamp,
    explorerUrl,
    formatOid,
    formatTimestamp,
    gravatarURL,
    pluralize,
  } from "@app/lib/utils";

  import Button from "@app/components/Button.svelte";
  import DiffActions from "@app/components/DiffActions.svelte";
  import DiffOptionsButton from "@app/components/DiffOptionsButton.svelte";
  import DiffStatBadge from "@app/components/DiffStatBadge.svelte";
  import ExternalLink from "@app/components/ExternalLink.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import JobCob from "@app/components/JobCob.svelte";
  import PierreDiff from "@app/components/PierreDiff.svelte";
  import PierreTree from "@app/components/PierreTree.svelte";
  import Topbar from "@app/components/Topbar.svelte";

  import Layout from "./Layout.svelte";

  interface Props {
    repo: RepoInfo;
    commit: Commit;
    diff: Diff;
    // Unified patch text feeding the rendered diff, fetched in the route loader
    // (see `loadRepoCommit`) so it lands in parallel with the structured diff.
    patch: string;
  }

  const { repo, commit, diff, patch }: Props = $props();

  let diffView = $state<{
    scrollToFile: (path: string) => void;
    setAllCollapsed: (collapsed: boolean) => void;
  }>();
  let allCollapsed = $state(false);

  // The new-side path Pierre keys each file by — renames and copies use the new
  // path, everything else its only path.
  function newSidePath(file: FileDiff): string {
    return file.status === "moved" || file.status === "copied"
      ? file.newPath
      : file.path;
  }

  // Changed files as tree paths + git status, both derived from the diff.
  const changedFiles = $derived(
    diff.files.map((file): GitStatusEntry => {
      const path = newSidePath(file);
      switch (file.status) {
        case "added":
          return { path, status: "added" };
        case "deleted":
          return { path, status: "deleted" };
        case "modified":
          return { path, status: "modified" };
        case "moved":
          return { path, status: "renamed" };
        case "copied":
          return { path, status: "added" };
      }
    }),
  );
  const treePaths = $derived(changedFiles.map(file => file.path));

  // Per-file metadata keyed by new-side path, built in one pass over the diff:
  //  - `notes`: files with no renderable text diff (binary blobs, empty/
  //    mode-only changes). Pierre has no binary concept and renders these as an
  //    empty body with a dead expand caret, so we show a note and drop the caret
  //    instead.
  //  - `statuses`: raw change status, driving the "Added"/"Deleted"/"Moved"/
  //    "Copied" label in the file header.
  const fileMeta = $derived.by(() => {
    /* eslint-disable svelte/prefer-svelte-reactivity -- rebuilt fresh each derivation, never mutated in place */
    const notes = new Map<string, FileNote>();
    const statuses = new Map<string, FileStatus>();
    /* eslint-enable svelte/prefer-svelte-reactivity */
    for (const file of diff.files) {
      const path = newSidePath(file);
      statuses.set(path, file.status);
      if (file.diff.type === "binary") {
        notes.set(path, "binary");
      } else if (file.diff.type === "empty") {
        notes.set(path, "empty");
      }
    }
    return { notes, statuses };
  });
  const fileNotes = $derived(fileMeta.notes);
  const fileStatuses = $derived(fileMeta.statuses);

  // Fetch a file's full old/new contents on demand (for diff expansion). The
  // old side comes from the commit's first parent; added/deleted files only
  // have one side. Reuses the existing `repo_blob` command — no backend change.
  async function fetchBlob(path: string, sha: string): Promise<string> {
    const blob = await invoke<Blob>("repo_blob", { rid: repo.rid, path, sha });
    return blob.binary ? "" : blob.content;
  }

  async function loadFullFile(
    path: string,
  ): Promise<{ oldContents: string; newContents: string }> {
    const parent = commit.parents[0];
    const file = diff.files.find(entry => newSidePath(entry) === path);
    let oldContents = "";
    let newContents = "";
    if (!file) {
      return { oldContents, newContents };
    }
    switch (file.status) {
      case "added":
        newContents = await fetchBlob(file.path, commit.id);
        break;
      case "deleted":
        if (parent) {
          oldContents = await fetchBlob(file.path, parent);
        }
        break;
      case "modified":
        newContents = await fetchBlob(file.path, commit.id);
        if (parent) {
          oldContents = await fetchBlob(file.path, parent);
        }
        break;
      case "moved":
      case "copied":
        newContents = await fetchBlob(file.newPath, commit.id);
        if (parent) {
          oldContents = await fetchBlob(file.oldPath, parent);
        }
        break;
    }
    return { oldContents, newContents };
  }
</script>

<style>
  .page {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }
  .body {
    display: flex;
    flex: 1;
    min-height: 0;
  }
  .tree-col {
    width: 16.5rem;
    flex: none;
    min-height: 0;
    border-right: 1px solid var(--color-border-subtle);
    padding-top: 0.5rem;
  }
  .breadcrumb {
    display: flex;
    align-items: center;
    gap: 0.375rem;
  }
  .topbar-right {
    margin-left: auto;
    display: flex;
    align-items: center;
  }
  .breadcrumb-link {
    cursor: pointer;
    background: none;
    border: none;
    padding: 0;
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
  }
  .breadcrumb-link:hover {
    color: var(--color-text-primary);
  }
  .meta {
    padding: 1rem 1rem 0.5rem;
  }
  .meta-header {
    display: flex;
    gap: 0.75rem;
    align-items: flex-start;
    justify-content: space-between;
    padding: 0 0 1rem;
    flex-wrap: wrap;
  }
  .meta-title {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    min-width: 0;
  }
  .summary {
    font: var(--txt-body-l-semibold);
    overflow-wrap: anywhere;
  }
  .summary-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    align-items: center;
    /* Reserve the JobCob widget's height (2rem) so the row doesn't shift when
       the job status loads in asynchronously. */
    min-height: 2rem;
    color: var(--color-text-secondary);
    font: var(--txt-body-m-regular);
  }
  .summary-author {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
  }
  .summary-avatar {
    width: 1rem;
    height: 1rem;
    border-radius: 999px;
    flex: none;
  }
  .summary-timestamp {
    color: var(--color-text-quaternary);
  }
  .summary-message {
    white-space: pre-wrap;
    margin: 0;
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
  }
  .summary-parents {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    align-items: center;
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
  }
  .summary-parents-label {
    font: inherit;
    color: inherit;
  }
  .parent-link {
    cursor: pointer;
    background: none;
    border: none;
    padding: 0;
  }
  .parent-link:hover {
    color: var(--color-text-primary);
  }
  .chips {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    flex-wrap: wrap;
    padding-top: 0.125rem;
  }
  .files-chip {
    padding: 0 0.5rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    height: 1.5rem;
    display: flex;
    align-items: center;
    font: var(--txt-code-regular);
    color: var(--color-text-secondary);
  }
</style>

{#snippet commitMeta()}
  <section class="meta">
    <div class="meta-header">
      <div class="meta-title">
        <div class="summary txt-selectable">
          {#if !commit.summary}
            <span class="txt-missing">No commit message</span>
          {:else}
            {commit.summary}
          {/if}
        </div>
        <div class="summary-meta">
          <span class="summary-author">
            <img
              class="summary-avatar"
              alt=""
              src={gravatarURL(commit.author.email)} />
            <span class="txt-selectable">{commit.author.name}</span>
          </span>
          committed
          <Id id={commit.id} clipboard={commit.id} />
          <span
            class="summary-timestamp"
            title={absoluteTimestamp(commit.committer.time * 1000)}>
            {formatTimestamp(commit.committer.time * 1000)}
          </span>
          <JobCob rid={repo.rid} commit={commit.id} />
        </div>
        <div class="summary-parents">
          <span class="summary-parents-label">
            {commit.parents.length === 1 ? "parent" : "parents"}
          </span>
          {#if commit.parents.length === 0}
            <span>Initial commit</span>
          {:else}
            {#each commit.parents as parent}
              <button
                class="parent-link txt-id"
                onclick={() => {
                  void router.push({
                    resource: "repo.commit",
                    rid: repo.rid,
                    commit: parent,
                  });
                }}>
                {formatOid(parent)}
              </button>
            {/each}
          {/if}
        </div>
        <pre class="summary-message txt-selectable">{commit.message
            .replace(commit.summary, "")
            .trim()}</pre>
      </div>
      <div class="chips">
        <div class="files-chip">
          {diff.stats.filesChanged}
          {pluralize("file", diff.stats.filesChanged)} changed
        </div>
        <DiffStatBadge stats={diff.stats} />
        <DiffActions
          text={() => Promise.resolve(patch)}
          fileName={`${formatOid(commit.id)}.diff`}
          title="Diff actions" />
      </div>
    </div>
  </section>
{/snippet}

<Layout selfScroll>
  <div class="page">
    <Topbar>
      <Button
        variant="naked"
        onclick={() => (diffOptions.showTree = !diffOptions.showTree)}
        title="Toggle file tree">
        <Icon
          name={diffOptions.showTree
            ? "sidebar-left-filled"
            : "sidebar-left"} />
      </Button>
      <div class="breadcrumb">
        <button
          class="breadcrumb-link"
          onclick={() =>
            router.push({
              resource: "repo.commits",
              rid: repo.rid,
            })}>
          All commits
        </button>
        <Icon name="chevron-right" />
        <Id id={commit.id} clipboard={commit.id} placement="bottom-start" />
        <ExternalLink
          href={explorerUrl(`${repo.rid}/commits/${commit.id}`)}
          title="Open in app.radicle.xyz" />
      </div>
      <div class="topbar-right">
        <Button
          variant="naked"
          onclick={() => {
            allCollapsed = !allCollapsed;
            diffView?.setAllCollapsed(allCollapsed);
          }}
          title={allCollapsed ? "Expand all files" : "Collapse all files"}>
          <Icon name={allCollapsed ? "expand-vertical" : "collapse-vertical"} />
        </Button>
        <Button
          variant="naked"
          onclick={() =>
            (diffOptions.diffStyle =
              diffOptions.diffStyle === "unified" ? "split" : "unified")}
          title={diffOptions.diffStyle === "unified"
            ? "Switch to split view"
            : "Switch to unified view"}>
          <Icon
            name={diffOptions.diffStyle === "unified"
              ? "diff-unified"
              : "diff-split"} />
        </Button>
        <DiffOptionsButton />
      </div>
    </Topbar>
    <div class="body">
      {#if diffOptions.showTree}
        <div class="tree-col">
          <PierreTree
            paths={treePaths}
            gitStatus={changedFiles}
            onSelect={path => diffView?.scrollToFile(path)} />
        </div>
      {/if}
      <!-- The metadata is passed as the header snippet so Pierre renders it
           inside its scroll container: it scrolls away, leaving only the
           sticky file headers pinned to the top. -->
      <PierreDiff
        bind:this={diffView}
        {patch}
        diffStyle={diffOptions.diffStyle}
        wordWrap={diffOptions.wordWrap}
        diffIndicators={diffOptions.indicators}
        lineDiffType={diffOptions.lineDiffType}
        {loadFullFile}
        {fileNotes}
        {fileStatuses}
        fileDiffText={path =>
          getDiffText(repo.rid, undefined, commit.id, 3, path)}
        cacheKeyPrefix={commit.id}
        header={commitMeta} />
    </div>
  </div>
</Layout>
