<script lang="ts">
  import type { Tree } from "@bindings/source/Tree";

  import Icon from "@app/components/Icon.svelte";
  import File from "@app/components/Source/File.svelte";
  import Folder from "@app/components/Source/Folder.svelte";
  import { getCurrentPath } from "@app/views/repo/RepoHome.svelte";

  interface Props {
    fetchTree: (path: string) => Promise<Tree>;
    fetchBlob: (path: string) => Promise<void>;
    currentPath: string;
    name: string;
    prefix: string;
  }

  const { name, fetchBlob, currentPath, prefix, fetchTree }: Props = $props();
  let expanded = $derived(getCurrentPath().indexOf(prefix) === 0);

  const treePromise = $derived(
    expanded ? fetchTree(prefix) : Promise.resolve(undefined),
  );
</script>

<style>
  .folder {
    cursor: pointer;
  }
  .folder:hover {
    background-color: var(--color-background-float);
  }
</style>

<div class="folder" style:padding-left="0.5rem">
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div
    class="global-flex txt-small"
    style:padding="0.25rem 0"
    onclick={() => (expanded = !expanded)}>
    <div class:txt-missing={!expanded}>
      <Icon name={expanded ? "folder-open" : "folder-closed"} />
    </div>
    {name}
  </div>
</div>
{#if expanded}
  {#await treePromise then tree}
    {#if tree}
      {#each tree.entries as entry (entry.path)}
        <div style:margin-left="1.5rem">
          {#if entry.kind === "tree"}
            <Folder
              {fetchTree}
              {fetchBlob}
              name={entry.name}
              {currentPath}
              prefix={`${entry.path}/`} />
          {:else if entry.kind === "blob"}
            <File
              name={entry.name}
              fetchBlob={() => fetchBlob(entry.path)}
              active={entry.path === getCurrentPath()} />
          {/if}
        </div>
      {/each}
    {/if}
  {/await}
{/if}
