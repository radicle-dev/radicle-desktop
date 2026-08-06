<script lang="ts">
  import type { CodeComments } from "@app/components/Diff.svelte";
  import type { Author } from "@bindings/cob/Author";
  import type { CodeLocation } from "@bindings/cob/thread/CodeLocation";
  import type { Embed } from "@bindings/cob/thread/Embed";
  import type { Thread } from "@bindings/cob/thread/Thread";
  import type { Config } from "@bindings/config/Config";
  import type { Diff as DiffType } from "@bindings/diff/Diff";
  import type { FileDiff } from "@bindings/diff/FileDiff";
  import type { Hunk } from "@bindings/diff/Hunk";

  import { cachedGetDiff } from "@app/lib/invoke";

  import Diff from "@app/components/Diff.svelte";
  import Icon from "@app/components/Icon.svelte";

  interface Props {
    rid: string;
    base: string;
    head: string;
    threads: Thread<CodeLocation>[];
    config: Config;
    repoDelegates: Author[];
    createComment?: (
      body: string,
      embeds: Embed[],
      replyTo?: string,
      location?: CodeLocation,
    ) => Promise<void>;
    editComment?: (
      commentId: string,
      body: string,
      embeds: Embed[],
    ) => Promise<void>;
    deleteComment?: (commentId: string) => Promise<void>;
    reactOnComment?: (
      commentId: string,
      authors: Author[],
      reaction: string,
    ) => Promise<void>;
  }

  const {
    rid,
    base,
    head,
    threads,
    config,
    repoDelegates,
    createComment,
    editComment,
    deleteComment,
    reactOnComment,
  }: Props = $props();

  // Per-thread hunk expand state, keyed by the thread's root comment id.
  let expandedThreads: Record<string, boolean> = $state({});

  const noop = () => Promise.resolve();

  // All threads passed to one instance belong to the same file, so the file
  // header is rendered once and each thread's hunk stacks beneath it.
  const path = $derived(threads[0]?.root.location?.path);

  function codeCommentsFor(thread: Thread<CodeLocation>): CodeComments {
    return {
      config,
      repoDelegates,
      rid,
      threads: [thread],
      canReply: Boolean(createComment),
      hideThreadFileHeader: true,
      createComment: createComment ?? noop,
      editComment: editComment ?? noop,
      deleteComment,
      reactOnComment,
    };
  }

  function findFileDiff(
    diff: DiffType,
    location: CodeLocation,
  ): FileDiff | undefined {
    return diff.files.find(file => {
      if (file.status === "moved" || file.status === "copied") {
        return file.oldPath === location.path || file.newPath === location.path;
      }
      return file.path === location.path;
    });
  }

  function findHunk(file: FileDiff, location: CodeLocation): Hunk | undefined {
    if (file.diff.type !== "plain") return undefined;
    const range = location.new ?? location.old;
    if (!range || range.type !== "lines") return undefined;
    const targetSide: "old" | "new" = location.new ? "new" : "old";

    return file.diff.hunks.find(hunk => {
      const hunkRange = hunk[targetSide];
      return (
        hunkRange.start <= range.range.start && hunkRange.end >= range.range.end
      );
    });
  }

  function sliceHunkAroundRange(
    hunk: Hunk,
    range: { start: number; end: number },
    side: "old" | "new",
    context: number,
  ): Hunk {
    const matches: number[] = [];
    hunk.lines.forEach((line, idx) => {
      let lineNo: number | undefined;
      if (line.type === "context") {
        lineNo = side === "new" ? line.lineNoNew : line.lineNoOld;
      } else if (side === "new" && line.type === "addition") {
        lineNo = line.lineNo;
      } else if (side === "old" && line.type === "deletion") {
        lineNo = line.lineNo;
      }
      if (lineNo !== undefined && lineNo >= range.start && lineNo < range.end) {
        matches.push(idx);
      }
    });

    if (matches.length === 0) return hunk;

    const startIdx = Math.max(0, matches[0] - context);
    const endIdx = Math.min(
      hunk.lines.length - 1,
      matches[matches.length - 1] + context,
    );

    if (startIdx === 0 && endIdx === hunk.lines.length - 1) return hunk;

    const slicedLines = hunk.lines.slice(startIdx, endIdx + 1);

    let oldStart: number | undefined;
    let oldEnd: number | undefined;
    let newStart: number | undefined;
    let newEnd: number | undefined;
    for (const line of slicedLines) {
      if (line.type === "context") {
        oldStart ??= line.lineNoOld;
        oldEnd = line.lineNoOld;
        newStart ??= line.lineNoNew;
        newEnd = line.lineNoNew;
      } else if (line.type === "addition") {
        newStart ??= line.lineNo;
        newEnd = line.lineNo;
      } else if (line.type === "deletion") {
        oldStart ??= line.lineNo;
        oldEnd = line.lineNo;
      }
    }

    const slicedOld =
      oldStart !== undefined && oldEnd !== undefined
        ? { start: oldStart, end: oldEnd + 1 }
        : hunk.old;
    const slicedNew =
      newStart !== undefined && newEnd !== undefined
        ? { start: newStart, end: newEnd + 1 }
        : hunk.new;

    const oldLen = slicedOld.end - slicedOld.start;
    const newLen = slicedNew.end - slicedNew.start;

    return {
      header: `@@ -${slicedOld.start},${oldLen} +${slicedNew.start},${newLen} @@`,
      lines: slicedLines,
      old: slicedOld,
      new: slicedNew,
    };
  }

  function sliceFileDiff(file: FileDiff, hunk: Hunk): FileDiff {
    if (file.diff.type !== "plain") return file;
    return {
      ...file,
      diff: { ...file.diff, hunks: [hunk] },
    };
  }
</script>

<style>
  .wrapper {
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-md);
    overflow: hidden;
  }
  .wrapper :global(.container) {
    padding-top: 0;
    padding-bottom: 0;
  }
  .file-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--color-border-subtle);
    background-color: var(--color-surface-canvas);
  }
  .file-path {
    flex: 1 1 0;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .thread + .thread {
    border-top: 1px solid var(--color-border-subtle);
  }
  .hunk-toolbar {
    display: flex;
    justify-content: flex-end;
    padding: 0.25rem 0.5rem 0;
  }
  .toggle {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    padding: 0.125rem;
    border-radius: var(--border-radius-sm);
    cursor: pointer;
    color: var(--color-text-tertiary);
  }
  .toggle:hover,
  .toggle:focus-visible {
    color: var(--color-text-primary);
    background-color: var(--color-surface-subtle);
  }
  .fallback {
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
  }
</style>

{#await cachedGetDiff(rid, { base, head, unified: 3, highlight: true })}
  <div class="fallback">Loading code…</div>
{:then diff}
  {@const location0 = threads[0]?.root.location}
  {#if path && location0}
    {@const file = findFileDiff(diff, location0)}
    {#if file}
      <div class="wrapper">
        <div class="file-header">
          <span class="file-path">{path}</span>
        </div>
        {#each threads as thread (thread.root.id)}
          {@const location = thread.root.location}
          {#if location}
            {@const hunk = findHunk(file, location)}
            {#if hunk}
              {@const codeRange = location.new ?? location.old}
              {@const targetSide = location.new ? "new" : "old"}
              {@const clampedHunk =
                codeRange && codeRange.type === "lines"
                  ? sliceHunkAroundRange(hunk, codeRange.range, targetSide, 3)
                  : hunk}
              {@const isClamped = clampedHunk !== hunk}
              {@const expanded = expandedThreads[thread.root.id] ?? false}
              {@const displayHunk = expanded ? hunk : clampedHunk}
              <div class="thread">
                {#if isClamped}
                  <div class="hunk-toolbar">
                    <button
                      class="toggle"
                      type="button"
                      title={expanded ? "Collapse hunk" : "Expand hunk"}
                      onclick={() =>
                        (expandedThreads = {
                          ...expandedThreads,
                          [thread.root.id]: !expanded,
                        })}>
                      <Icon
                        name={expanded
                          ? "collapse-vertical"
                          : "expand-vertical"} />
                    </button>
                  </div>
                {/if}
                <Diff
                  file={sliceFileDiff(file, displayHunk)}
                  {head}
                  codeComments={codeCommentsFor(thread)} />
              </div>
            {:else}
              <div class="fallback">Code unavailable for {location.path}</div>
            {/if}
          {/if}
        {/each}
      </div>
    {:else}
      <div class="fallback">Code unavailable for {path}</div>
    {/if}
  {/if}
{/await}
