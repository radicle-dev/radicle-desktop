<script lang="ts">
  import type { Diff } from "@bindings/diff/Diff";
  import type { CodeComments } from "./Diff.svelte";

  import FileDiffComponent from "./FileDiff.svelte";

  interface Props {
    codeComments?: CodeComments;
    diff: Diff;
    expanded?: boolean;
    head: string;
  }

  const { codeComments, diff, expanded = true, head }: Props = $props();
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
  {#each diff.files as file}
    <div class="diff">
      <FileDiffComponent {codeComments} {expanded} {file} {head} />
    </div>
  {/each}
</div>
