<script lang="ts">
  import type { Tree } from "@bindings/source/Tree";

  import FileTreeFile from "@app/components/FileTreeFile.svelte";
  import FileTreeFolder from "@app/components/FileTreeFolder.svelte";

  interface Props {
    tree: Tree;
    currentPath: string;
    fetchTree: (path: string) => Promise<Tree>;
    fetchBlob: (path: string) => Promise<void>;
  }

  const { currentPath, tree, fetchTree, fetchBlob }: Props = $props();
</script>

<div style:display="flex" style:flex-direction="column" style:gap="0.25rem">
  {#each tree.entries as entry (entry.name)}
    {#if entry.kind === "tree"}
      <FileTreeFolder
        name={entry.name}
        prefix={`${entry.path}/`}
        {currentPath}
        {fetchTree}
        {fetchBlob} />
    {:else}
      <FileTreeFile
        name={entry.name}
        fetchBlob={() => fetchBlob(entry.path)}
        active={entry.path === currentPath} />
    {/if}
  {/each}
</div>
