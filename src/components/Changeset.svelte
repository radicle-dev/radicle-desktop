<script lang="ts">
  import type { Diff } from "@bindings/diff/Diff";

  import FileDiff from "./Changeset/FileDiff.svelte";

  interface Props {
    diff: Diff;
    repoId: string;
  }

  const { diff }: Props = $props();
</script>

<style>
  .diff-list {
    display: flex;
    flex-direction: column;
  }
</style>

<div class="diff-list">
  {#each diff.files as file}
    <div style:margin-bottom="1rem">
      <FileDiff
        filePath={"path" in file ? file.path : file.newPath}
        oldFilePath={"oldPath" in file ? file.oldPath : undefined}
        fileDiff={file.diff}
        headerBadgeCaption={file.status} />
    </div>
  {/each}
</div>
