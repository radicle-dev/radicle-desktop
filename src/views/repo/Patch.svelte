<script lang="ts">
  import type { Diff } from "@bindings/diff/Diff";
  import type { Config } from "@bindings/config/Config";
  import type { PaginatedQuery } from "@bindings/cob/PaginatedQuery";
  import type { Patch } from "@bindings/cob/patch/Patch";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";
  import type { Revision } from "@bindings/cob/patch/Revision";
  import type { PatchStatus } from "./router";

  import { invoke } from "@app/lib/invoke";

  import CommentComponent from "@app/components/Comment.svelte";
  import CopyableId from "@app/components/CopyableId.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import InlineTitle from "@app/components/InlineTitle.svelte";
  import Layout from "./Layout.svelte";
  import Link from "@app/components/Link.svelte";
  import NodeId from "@app/components/NodeId.svelte";
  import PatchTeaser from "@app/components/PatchTeaser.svelte";
  import Sidebar from "@app/components/Sidebar.svelte";
  import Changeset from "@app/components/Changeset.svelte";

  interface Props {
    repo: RepoInfo;
    patch: Patch;
    patches: PaginatedQuery<Patch[]>;
    revisions: Revision[];
    config: Config;
    status: PatchStatus | undefined;
  }

  /* eslint-disable prefer-const */
  let { repo, patch, patches, revisions, config, status }: Props = $props();
  /* eslint-enable prefer-const */

  let items = $state(patches.content);
  let cursor = patches.cursor;
  let more = patches.more;

  $effect(() => {
    items = patches.content;
    cursor = patches.cursor;
    more = patches.more;
  });

  async function loadHighlightedDiff(rid: string, base: string, head: string) {
    return invoke<Diff>("get_diff", {
      rid,
      options: {
        base,
        head,
        unified: 5,
        highlight: true,
      },
    });
  }

  async function loadPatch(rid: string, patchId: string) {
    patch = await invoke<Patch>("patch_by_id", {
      rid: rid,
      id: patchId,
    });
    revisions = await invoke<Revision[]>("revisions_by_patch", {
      rid: rid,
      id: patchId,
    });
  }

  async function loadMoreSecondColumn() {
    if (more) {
      const p = await invoke<PaginatedQuery<Patch[]>>("list_patches", {
        rid: repo.rid,
        skip: cursor + 20,
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
  .title {
    font-size: var(--font-size-medium);
    font-weight: var(--font-weight-medium);
    -webkit-user-select: text;
    user-select: text;
    margin-bottom: 1rem;
  }
  .patch-list {
    margin-top: 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding-bottom: 1rem;
  }
  .content {
    padding: 1.5rem 1rem 1rem 0;
  }

  .patch-body {
    margin-top: 1rem;
    position: relative;
  }
  /* We put the background and clip-path in a separate element to prevent
     popovers being clipped in the main element. */
  .patch-body::after {
    position: absolute;
    z-index: -1;
    content: " ";
    background-color: var(--color-background-float);
    clip-path: var(--2px-corner-fill);
    width: 100%;
    height: 100%;
    top: 0;
  }
  .breadcrumbs {
    display: flex;
    gap: 0.5rem;
    font-size: var(--font-size-tiny);
    font-weight: var(--font-weight-semibold);
    align-items: center;
    min-height: 1.5rem;
    width: 100%;
    margin-bottom: 1rem;
    color: var(--color-foreground-dim);
  }
</style>

<Layout {loadMoreSecondColumn} publicKey={config.publicKey}>
  {#snippet headerCenter()}
    <CopyableId id={patch.id} />
  {/snippet}

  {#snippet sidebar()}
    <Sidebar activeTab={{ type: "patches", status }} rid={repo.rid} />
  {/snippet}

  {#snippet secondColumn()}
    <div
      style:height="34px"
      class="global-flex txt-medium"
      style:font-weight="var(--font-weight-medium)">
      Patches
    </div>
    <div class="patch-list">
      {#each items as p}
        <PatchTeaser
          compact
          {loadPatch}
          patch={p}
          rid={repo.rid}
          {status}
          selected={patch && p.id === patch.id} />
      {/each}
    </div>
  {/snippet}

  <div class="content">
    <div class="title">
      <InlineTitle content={patch.title} fontSize="medium" />
    </div>

    <div class="breadcrumbs">
      <Link route={{ resource: "home" }}>
        <NodeId
          publicKey={config.publicKey}
          alias={config.alias}
          styleFontFamily="var(--font-family-sans-serif)"
          styleFontSize="var(--font-size-tiny)" />
      </Link>
      <Icon name="chevron-right" />
      <Link
        route={{ resource: "repo.patches", rid: repo.rid, status: "open" }}
        styleColor="var(--color-foreground-dim)">
        {project.data.name}
      </Link>
      <Icon name="chevron-right" />
      <Link
        route={{ resource: "repo.patches", rid: repo.rid, status: "open" }}
        styleColor="var(--color-foreground-dim)">
        Patches
      </Link>
      <Icon name="chevron-right" />
      {patch.title}
    </div>

    <div class="txt-small patch-body">
      <CommentComponent
        caption="opened"
        rid={repo.rid}
        id={patch.id}
        lastEdit={revisions[0].description.length > 1
          ? revisions[0].description.at(-1)
          : undefined}
        author={revisions[0].author}
        reactions={revisions[0].reactions}
        timestamp={revisions[0].description.slice(-1)[0].timestamp}
        body={revisions[0].description.slice(-1)[0].body}>
      </CommentComponent>
    </div>
    <div class="txt-small" style:margin-top="1rem">Revisions</div>
    {#each revisions as revision}
      <div><Id id={revision.id} variant="oid" /></div>
      {#await loadHighlightedDiff(repo.rid, revision.base, revision.head) then diff}
        <Changeset {diff} repoId={repo.rid} />
      {/await}
    {/each}
  </div>
</Layout>
