<script lang="ts">
  import type { CodeComments } from "@app/components/Diff.svelte";
  import type { Diff } from "@bindings/diff/Diff";

  import FileDiffComponent from "@app/components/FileDiff.svelte";

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
