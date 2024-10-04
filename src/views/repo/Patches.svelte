<script lang="ts">
  import type { Config } from "@bindings/Config";
  import type { PaginatedQuery } from "@bindings/PaginatedQuery";
  import type { Patch } from "@bindings/Patch";
  import type { PatchStatus } from "./router";
  import type { RepoInfo } from "@bindings/RepoInfo";

  import { invoke } from "@tauri-apps/api/core";

  import Layout from "./Layout.svelte";
  import Border from "@app/components/Border.svelte";
  import CopyableId from "@app/components/CopyableId.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Link from "@app/components/Link.svelte";
  import NodeId from "@app/components/NodeId.svelte";
  import PatchTeaser from "@app/components/PatchTeaser.svelte";
  import RepoHeader from "@app/components/RepoHeader.svelte";

  export let repo: RepoInfo;
  export let patches: PaginatedQuery<Patch[]>;
  export let config: Config;
  export let status: PatchStatus | undefined = undefined;

  $: items = patches.content;
  $: more = patches.more;
  $: cursor = patches.cursor;

  async function loadMore() {
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

  $: project = repo.payloads["xyz.radicle.project"]!;
</script>

<style>
  .list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 0 1rem 1rem 1rem;
  }
</style>

<Layout {loadMore}>
  <svelte:fragment slot="breadcrumbs">
    <Link route={{ resource: "home" }}>
      <NodeId
        nodeId={config.publicKey}
        alias={config.alias}
        styleFontFamily="var(--font-family-sans-serif)"
        styleFontSize="var(--font-size-tiny)" />
    </Link>
    <Link route={{ resource: "repo.issues", rid: repo.rid, status: "open" }}>
      <div class="global-flex">
        <Icon name="chevron-right" />
        {project.data.name}
      </div>
    </Link>
    <Icon name="chevron-right" />
    Patches
  </svelte:fragment>
  <svelte:fragment slot="header-center">
    <CopyableId id={repo.rid} />
  </svelte:fragment>

  <svelte:fragment slot="sidebar">
    <Border
      hoverable={false}
      variant="ghost"
      styleWidth="100%"
      styleHeight="32px">
      <RepoHeader
        {repo}
        selfDid={`did:key:${config.publicKey}`}
        emphasizedTitle={false} />
    </Border>

    <div class="global-flex txt-small" style:margin="0.5rem 0">
      <Link
        variant="tab"
        route={{ resource: "repo.issues", rid: repo.rid, status: "all" }}>
        <div class="global-flex"><Icon name="issue" />Issues</div>
        <div class="global-counter">
          {project.meta.issues.open + project.meta.issues.closed}
        </div>
      </Link>
    </div>
    <div class="global-flex txt-small" style:margin="0.5rem 0">
      <Link
        variant={status === undefined ? "active" : "tab"}
        route={{ resource: "repo.patches", rid: repo.rid }}>
        <div class="global-flex"><Icon name="patch" />Patches</div>
        <div class="global-counter">
          {project.meta.patches.draft +
            project.meta.patches.open +
            project.meta.patches.archived +
            project.meta.patches.merged}
        </div>
      </Link>
    </div>
    <div class="global-flex txt-small global-tab">
      <Link
        variant={status === "draft" ? "active" : "tab"}
        route={{
          resource: "repo.patches",
          rid: repo.rid,
          status: "draft",
        }}>
        Draft <div class="global-counter">
          {project.meta.patches.draft}
        </div>
      </Link>
      <Link
        variant={status === "open" ? "active" : "tab"}
        route={{ resource: "repo.patches", rid: repo.rid, status: "open" }}>
        Open <div class="global-counter">
          {project.meta.patches.open}
        </div>
      </Link>
      <Link
        variant={status === "archived" ? "active" : "tab"}
        route={{
          resource: "repo.patches",
          rid: repo.rid,
          status: "archived",
        }}>
        Archived <div class="global-counter">
          {project.meta.patches.archived}
        </div>
      </Link>
      <Link
        variant={status === "merged" ? "active" : "tab"}
        route={{
          resource: "repo.patches",
          rid: repo.rid,
          status: "merged",
        }}>
        Merged <div class="global-counter">
          {project.meta.patches.merged}
        </div>
      </Link>
    </div>
  </svelte:fragment>

  <div class="list">
    {#each items as patch}
      <PatchTeaser rid={repo.rid} {patch} />
    {/each}

    {#if patches.content.length === 0}
      <div class="txt-missing txt-small">
        {#if status === undefined}
          No patches.
        {:else}
          No {status} patches.
        {/if}
      </div>
    {/if}
  </div>
</Layout>
