<script lang="ts">
  import type { CodeComments } from "@app/components/Diff.svelte";
  import type { Author } from "@bindings/cob/Author";
  import type { CodeLocation } from "@bindings/cob/thread/CodeLocation";
  import type { Thread } from "@bindings/cob/thread/Thread";
  import type { Config } from "@bindings/config/Config";
  import type { Diff as DiffType } from "@bindings/diff/Diff";
  import type { FileDiff } from "@bindings/diff/FileDiff";
  import type { Hunk } from "@bindings/diff/Hunk";

  import { cachedGetDiff } from "@app/lib/invoke";

  import Diff from "@app/components/Diff.svelte";

  interface Props {
    rid: string;
    base: string;
    head: string;
    thread: Thread<CodeLocation>;
    config: Config;
    repoDelegates: Author[];
  }

  const { rid, base, head, thread, config, repoDelegates }: Props = $props();

  const noop = () => Promise.resolve();

  const codeComments: CodeComments = {
    config,
    repoDelegates,
    rid,
    threads: [thread],
    canReply: false,
    hideThreadFileHeader: true,
    createComment: noop,
    editComment: noop,
  };

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
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--color-border-subtle);
    background-color: var(--color-surface-canvas);
  }
  .fallback {
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
  }
</style>

{#await cachedGetDiff(rid, { base, head, unified: 3, highlight: true })}
  <div class="fallback">Loading code…</div>
{:then diff}
  {@const location = thread.root.location}
  {#if location}
    {@const file = findFileDiff(diff, location)}
    {#if file}
      {@const hunk = findHunk(file, location)}
      {#if hunk}
        <div class="wrapper">
          <div class="file-header">{location.path}</div>
          <Diff file={sliceFileDiff(file, hunk)} {head} {codeComments} />
        </div>
      {:else}
        <div class="fallback">Code unavailable for {location.path}</div>
      {/if}
    {:else}
      <div class="fallback">Code unavailable for {location.path}</div>
    {/if}
  {/if}
{/await}
