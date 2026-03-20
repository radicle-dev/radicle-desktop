<script lang="ts">
  import type { Tree } from "@bindings/source/Tree";

  import FileTreeFile from "@app/components/FileTreeFile.svelte";
  import FileTreeFolder from "@app/components/FileTreeFolder.svelte";
  import Icon from "@app/components/Icon.svelte";

  interface Props {
    fetchTree: (path: string) => Promise<Tree>;
    fetchBlob: (path: string) => Promise<void>;
    currentPath: string;
    name: string;
    prefix: string;
    indent?: number;
  }

  const {
    name,
    fetchBlob,
    currentPath,
    prefix,
    fetchTree,
    indent = 0.5,
  }: Props = $props();
  let expanded = $derived(currentPath.indexOf(prefix) === 0);

  const treePromise = $derived(
    expanded ? fetchTree(prefix) : Promise.resolve(undefined),
  );
</script>

<style>
  .folder {
    cursor: pointer;
    width: 100%;
    border-radius: var(--border-radius-sm);
  }
  .folder:hover {
    background-color: var(--color-surface-subtle);
  }
</style>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  class="folder"
  style:padding-left="{indent}rem"
  style:padding-right="0.5rem"
  onclick={() => (expanded = !expanded)}>
  <div class="global-flex txt-body-m-regular" style:padding="0.25rem 0">
    <div class:txt-missing={!expanded}>
      <Icon name={expanded ? "folder-open" : "folder"} />
    </div>
    {name}
  </div>
</div>
{#if expanded}
  {#await treePromise then tree}
    {#if tree}
      <div
        style:display="flex"
        style:flex-direction="column"
        style:gap="0.25rem">
        {#each tree.entries as entry (entry.path)}
          {#if entry.kind === "tree"}
            <FileTreeFolder
              {fetchTree}
              {fetchBlob}
              name={entry.name}
              {currentPath}
              prefix={`${entry.path}/`}
              indent={indent + 1.5} />
          {:else if entry.kind === "blob"}
            <FileTreeFile
              name={entry.name}
              fetchBlob={() => fetchBlob(entry.path)}
              active={entry.path === currentPath}
              indent={indent + 1.5} />
          {/if}
        {/each}
      </div>
    {/if}
  {/await}
{/if}
