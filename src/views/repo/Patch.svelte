<script lang="ts">
  import type { Config } from "@bindings/config/Config";
  import type { Patch } from "@bindings/cob/patch/Patch";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";
  import type { Revision } from "@bindings/cob/patch/Revision";

  import {
    authorForNodeId,
    formatTimestamp,
    patchStatusColor,
  } from "@app/lib/utils";
  import { invoke } from "@tauri-apps/api/core";

  import Border from "@app/components/Border.svelte";
  import CopyableId from "@app/components/CopyableId.svelte";
  import Icon from "@app/components/Icon.svelte";
  import InlineTitle from "@app/components/InlineTitle.svelte";
  import Layout from "./Layout.svelte";
  import Link from "@app/components/Link.svelte";
  import NodeId from "@app/components/NodeId.svelte";
  import Markdown from "@app/components/Markdown.svelte";
  import Id from "@app/components/Id.svelte";

  export let repo: RepoInfo;
  export let patch: Patch;
  export let patches: Patch[];
  export let revisions: Revision[];
  export let config: Config;

  $: void invoke("get_diff", {
    rid: repo.rid,
    options: {
      base: revisions[0].base,
      head: revisions[0].head,
      unified: 10,
    },
  }).then(console.log);

  $: project = repo.payloads["xyz.radicle.project"]!;
</script>

<style>
  .title {
    font-size: var(--font-size-medium);
    font-weight: var(--font-weight-medium);
    -webkit-user-select: text;
    user-select: text;
    margin-bottom: 1rem;
    margin-top: 0.35rem;
  }
  .patch-teaser {
    max-width: 11rem;
    white-space: nowrap;
  }
  .patch-list {
    margin-top: 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding-bottom: 1rem;
  }
  .content {
    padding: 0 1rem 1rem 1rem;
  }

  .body {
    background-color: var(--color-background-float);
    padding: 1rem;
  }
</style>

<Layout>
  <svelte:fragment slot="breadcrumbs">
    <Link route={{ resource: "home" }}>
      <NodeId
        publicKey={config.publicKey}
        alias={config.alias}
        styleFontFamily="var(--font-family-sans-serif)"
        styleFontSize="var(--font-size-tiny)" />
    </Link>
    <Link route={{ resource: "repo.patches", rid: repo.rid, status: "open" }}>
      <div class="global-flex">
        <Icon name="chevron-right" />
        {project.data.name}
      </div>
    </Link>
    <Icon name="chevron-right" />
    Patches
  </svelte:fragment>

  <svelte:fragment slot="header-center">
    <CopyableId id={patch.id} />
  </svelte:fragment>

  <svelte:fragment slot="sidebar">
    <Border
      hoverable={false}
      variant="ghost"
      styleWidth="100%"
      styleHeight="32px">
      <div style:margin-left="0.5rem">
        <Icon name="patch" />
      </div>
      <span class="txt-small txt-semibold">Patches</span>
      <div class="global-flex txt-small" style:margin-left="auto">
        <div
          class="global-counter"
          style:padding="0 6px"
          style:background-color="var(--color-fill-ghost)"
          style:gap="4px">
          {project.meta.patches.draft +
            project.meta.patches.open +
            project.meta.patches.merged +
            project.meta.patches.archived}
        </div>
      </div>
    </Border>

    <div class="patch-list">
      {#each patches as sidebarPatch}
        <Link
          variant="tab"
          route={{
            resource: "repo.patch",
            rid: repo.rid,
            patch: sidebarPatch.id,
          }}>
          <div class="global-flex">
            <div
              style:color={patchStatusColor[sidebarPatch.state.status]}
              style:margin-left="2px">
              <Icon name="patch" />
            </div>
            <span class="txt-small patch-teaser txt-overflow">
              <InlineTitle content={sidebarPatch.title} fontSize="small" />
            </span>
          </div>
        </Link>
      {/each}
    </div>
  </svelte:fragment>

  <div class="content">
    <div class="title">
      <InlineTitle content={patch.title} fontSize="medium" />
    </div>
    <div class="txt-small body">
      {#if revisions[0].description.slice(-1)[0].body !== ""}
        <Markdown
          rid={repo.rid}
          breaks
          content={revisions[0].description.slice(-1)[0].body} />
      {:else}
        <span class="txt-missing" style:line-height="1.625rem">
          No description.
        </span>
      {/if}
      <div class="global-flex txt-small" style:margin-top="1.5rem">
        <NodeId {...authorForNodeId(patch.author)} />
        opened
        <Id id={patch.id} variant="oid" />
        {formatTimestamp(patch.timestamp)}
      </div>
    </div>
    <div class="txt-small" style:margin-top="1rem">Revisions</div>
    {#each revisions as revision}
      <Id id={revision.id} variant="oid" />
    {/each}
  </div>
</Layout>
