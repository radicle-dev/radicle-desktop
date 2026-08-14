<script lang="ts">
  import type { Author } from "@bindings/cob/Author";
  import type { CodeLocation } from "@bindings/cob/thread/CodeLocation";
  import type { Embed } from "@bindings/cob/thread/Embed";
  import type { Thread } from "@bindings/cob/thread/Thread";
  import type { Config } from "@bindings/config/Config";

  import type { CodeComments } from "@app/lib/codeComments";
  import { diffOptions } from "@app/lib/diffOptions.svelte";
  import { cachedGetDiffText } from "@app/lib/invoke";
  import { sliceForRange } from "@app/lib/patchSlice";

  import PierreSnippet from "@app/components/PierreSnippet.svelte";

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

  /// Which lines of which side a comment is about, in the numbering
  /// `sliceForRange` matches on.
  function rangeOf(
    location: CodeLocation,
  ): { side: "old" | "new"; start: number; end: number } | undefined {
    const range = location.new ?? location.old;
    if (!range) return undefined;
    const side = location.new ? "new" : "old";
    if (range.type === "chars") {
      return { side, start: range.line, end: range.line + 1 };
    }
    return { side, start: range.range.start, end: range.range.end };
  }
</script>

<style>
  .wrapper {
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-md);
    overflow: hidden;
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
  .fallback {
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
  }
</style>

{#if path}
  <!-- The file's own patch text, which is all the renderer needs; the structured
       diff is not fetched here at all. -->
  {#await cachedGetDiffText(rid, base, head, 3, path)}
    <div class="fallback">Loading code…</div>
  {:then filePatch}
    <div class="wrapper">
      <div class="file-header">
        <span class="file-path">{path}</span>
      </div>
      {#each threads as thread (thread.root.id)}
        {@const location = thread.root.location}
        {#if location}
          {@const range = rangeOf(location)}
          {@const slice =
            range &&
            sliceForRange(filePatch, range.side, range.start, range.end)}
          {#if slice}
            <div class="thread">
              <PierreSnippet
                patch={slice}
                {path}
                cacheKey={`snippet:${head}:${thread.root.id}`}
                threads={[thread]}
                codeComments={codeCommentsFor(thread)}
                diffIndicators={diffOptions.indicators}
                lineDiffType={diffOptions.lineDiffType} />
            </div>
          {:else}
            <div class="fallback">Code unavailable for {location.path}</div>
          {/if}
        {/if}
      {/each}
    </div>
  {/await}
{/if}
