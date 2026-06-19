<script lang="ts">
  import type { Tree } from "@bindings/source/Tree";

  import { SvelteMap } from "svelte/reactivity";

  import FileTreeFile from "@app/components/FileTreeFile.svelte";
  import FileTreeFolder from "@app/components/FileTreeFolder.svelte";
  import VirtualList from "@app/components/VirtualList.svelte";

  interface Props {
    tree: Tree;
    currentPath: string;
    fetchTree: (path: string) => Promise<Tree>;
    fetchBlob: (path: string) => Promise<void>;
  }

  const { currentPath, tree, fetchTree, fetchBlob }: Props = $props();

  // Manual expand/collapse overrides, keyed by folder prefix. Held here
  // rather than in FileTreeFolder so the state survives the virtualizer
  // unmounting off-screen rows; without an override a folder is expanded
  // when it contains the open file.
  const expandedOverrides = new SvelteMap<string, boolean>();
  $effect(() => {
    // Navigating to another file resets every folder to its default, the
    // same behaviour the per-folder writable $derived had before the state
    // was hoisted.
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    currentPath;
    expandedOverrides.clear();
  });
  function isExpanded(prefix: string): boolean {
    return expandedOverrides.get(prefix) ?? currentPath.indexOf(prefix) === 0;
  }
  function toggleExpanded(prefix: string) {
    expandedOverrides.set(prefix, !isExpanded(prefix));
  }
</script>

<VirtualList
  items={tree.entries}
  getKey={entry => entry.path}
  estimatedItemSize={28}>
  {#snippet row(entry)}
    {#if entry.kind === "tree"}
      <FileTreeFolder
        name={entry.name}
        prefix={`${entry.path}/`}
        {currentPath}
        {isExpanded}
        {toggleExpanded}
        {fetchTree}
        {fetchBlob} />
    {:else}
      <FileTreeFile
        name={entry.name}
        fetchBlob={() => fetchBlob(entry.path)}
        active={entry.path === currentPath} />
    {/if}
  {/snippet}
</VirtualList>
