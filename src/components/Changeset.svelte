<script lang="ts">
  import type { CodeComments } from "@app/components/Diff.svelte";
  import type { Diff } from "@bindings/diff/Diff";

  import { isIgnoredFile, isLargeFile } from "@app/lib/ignoredFiles";

  import FileDiffComponent from "@app/components/FileDiff.svelte";

  interface Props {
    codeComments?: CodeComments;
    diff: Diff;
    expanded?: boolean;
    head: string;
    rid: string;
    draftReviewId?: string;
  }

  const {
    codeComments,
    diff,
    expanded = true,
    head,
    rid,
    draftReviewId,
  }: Props = $props();

  // Rendering every file's highlighted diff at once blocks the main thread on
  // mount (a large patch is thousands of {@html} lines). Reveal files in small
  // batches across animation frames so the tab appears immediately and stays
  // responsive while the rest stream in.
  const INITIAL_BATCH = 8;
  const FRAME_BATCH = 6;
  let visibleCount = $state(INITIAL_BATCH);
  $effect(() => {
    const total = diff.files.length;
    visibleCount = Math.min(INITIAL_BATCH, total);
    if (visibleCount >= total) return;
    let raf = 0;
    const step = () => {
      visibleCount = Math.min(visibleCount + FRAME_BATCH, total);
      if (visibleCount < total) raf = requestAnimationFrame(step);
    };
    raf = requestAnimationFrame(step);
    return () => cancelAnimationFrame(raf);
  });
</script>

<style>
  .diff-list {
    display: flex;
    flex-direction: column;
  }

  .diff:not(:last-of-type) {
    margin-bottom: 1rem;
  }
</style>

<div class="diff-list">
  {#each diff.files.slice(0, visibleCount) as file}
    <div class="diff">
      <FileDiffComponent
        {codeComments}
        expanded={expanded && !isIgnoredFile(file) && !isLargeFile(file)}
        {file}
        {head}
        {rid}
        {draftReviewId} />
    </div>
  {/each}
</div>
