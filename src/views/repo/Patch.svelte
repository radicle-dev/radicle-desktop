<script lang="ts">
  import type { Config } from "@bindings/config/Config";
  import type { Patch } from "@bindings/cob/patch/Patch";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";
  import type { Revision } from "@bindings/cob/patch/Revision";

  import { authorForNodeId, formatTimestamp } from "@app/lib/utils";

  import CopyableId from "@app/components/CopyableId.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import InlineTitle from "@app/components/InlineTitle.svelte";
  import Layout from "./Layout.svelte";
  import Link from "@app/components/Link.svelte";
  import Markdown from "@app/components/Markdown.svelte";
  import NodeId from "@app/components/NodeId.svelte";
  import PatchTeaser from "@app/components/PatchTeaser.svelte";
  import Sidebar from "@app/components/Sidebar.svelte";

  interface Props {
    repo: RepoInfo;
    patch: Patch;
    patches: Patch[];
    revisions: Revision[];
    config: Config;
  }

  const { repo, patch, patches, revisions, config }: Props = $props();

  const project = $derived(repo.payloads["xyz.radicle.project"]!);
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
  .patch-list {
    margin-top: 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding-bottom: 1rem;
  }
  .content {
    padding: 0 1rem 1rem 0;
  }

  .body {
    background-color: var(--color-background-float);
    padding: 1rem;
  }
</style>

<Layout>
  {#snippet breadcrumbs()}
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
    <Link route={{ resource: "repo.patches", rid: repo.rid, status: "open" }}>
      Patches
    </Link>
    <Icon name="chevron-right" />
    {patch.title}
  {/snippet}

  {#snippet headerCenter()}
    <CopyableId id={patch.id} />
  {/snippet}

  {#snippet sidebar()}
    <Sidebar activeTab="patches" rid={repo.rid} />
  {/snippet}

  {#snippet secondColumn()}
    <div
      style:height="34px"
      class="global-flex txt-medium"
      style:font-weight="var(--font-weight-medium)">
      Patches
    </div>
    <div class="patch-list">
      {#each patches as p}
        <PatchTeaser
          compact
          patch={p}
          rid={repo.rid}
          selected={patch && p.id === patch.id} />
      {/each}
    </div>
  {/snippet}

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
