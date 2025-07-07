<script lang="ts">
  import type { Tree } from "@bindings/source/Tree";

  import File from "@app/components/Source/File.svelte";
  import Folder from "@app/components/Source/Folder.svelte";
  import { getCurrentPath } from "@app/views/repo/RepoHome.svelte";

  interface Props {
    tree: Tree;
    path?: string;
    fetchTree: (path: string) => Promise<Tree>;
    fetchBlob: (path: string) => Promise<void>;
  }

  const { path = "", tree, fetchTree, fetchBlob }: Props = $props();
</script>

<div>
  {#each tree.entries as entry (entry.name)}
    {#if entry.kind === "tree"}
      <Folder
        name={entry.name}
        prefix={`${entry.path}/`}
        currentPath={path}
        {fetchTree}
        {fetchBlob} />
    {:else}
      <File
        name={entry.name}
        fetchBlob={() => fetchBlob(entry.path)}
        active={entry.path === getCurrentPath()} />
    {/if}
  {/each}
</div>
