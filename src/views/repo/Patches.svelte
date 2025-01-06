<script lang="ts">
  import type { Config } from "@bindings/config/Config";
  import type { PaginatedQuery } from "@bindings/cob/PaginatedQuery";
  import type { Patch } from "@bindings/cob/patch/Patch";
  import type { PatchStatus } from "./router";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import { invoke } from "@app/lib/invoke";

  import CopyableId from "@app/components/CopyableId.svelte";
  import Layout from "./Layout.svelte";
  import PatchTeaser from "@app/components/PatchTeaser.svelte";
  import PatchesSecondColumn from "@app/components/PatchesSecondColumn.svelte";
  import Sidebar from "@app/components/Sidebar.svelte";

  interface Props {
    repo: RepoInfo;
    patches: PaginatedQuery<Patch[]>;
    config: Config;
    status: PatchStatus | undefined;
  }

  const { repo, patches, config, status }: Props = $props();

  let items = $state(patches.content);
  let cursor = patches.cursor;
  let more = patches.more;

  $effect(() => {
    items = patches.content;
    cursor = patches.cursor;
    more = patches.more;
  });

  async function loadMoreContent() {
    if (more) {
      const p = await invoke<PaginatedQuery<Patch[]>>("list_patches", {
        rid: repo.rid,
        skip: cursor + 20,
        status,
        take: 20,
      });

      cursor = p.cursor;
      more = p.more;
      items = [...items, ...p.content];
    }
  }

  const project = $derived(repo.payloads["xyz.radicle.project"]!);
</script>

<style>
  .list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 0 1rem 1rem 0;
  }
  .header {
    font-weight: var(--font-weight-medium);
    font-size: var(--font-size-medium);
    padding: 1rem 1rem 0.5rem 1rem;
    display: flex;
    align-items: center;
    height: 58px;
  }
</style>

<Layout
  {loadMoreContent}
  hideSidebar
  styleSecondColumnOverflow="visible"
  publicKey={config.publicKey}>
  {#snippet headerCenter()}
    <CopyableId id={repo.rid} />
  {/snippet}

  {#snippet sidebar()}
    <Sidebar activeTab={{ type: "patches", status }} rid={repo.rid} />
  {/snippet}

  {#snippet secondColumn()}
    <div style:margin-left="1rem" style:height="100%">
      <PatchesSecondColumn {project} {status} {repo} />
    </div>
  {/snippet}

  <div class="header">Patches</div>

  <div class="list">
    {#each items as patch}
      <PatchTeaser rid={repo.rid} {patch} {status} />
    {/each}

    {#if patches.content.length === 0}
      <div class="txt-missing txt-small" style:margin-left="1rem">
        {#if status === undefined}
          No patches.
        {:else}
          No {status} patches.
        {/if}
      </div>
    {/if}
  </div>
</Layout>
