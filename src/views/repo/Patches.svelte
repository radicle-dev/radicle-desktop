<script lang="ts">
  import type { Config } from "@bindings/Config";
  import type { Patch } from "@bindings/Patch";
  import type { PatchStatus } from "./router";
  import type { RepoInfo } from "@bindings/RepoInfo";

  import { formatOid } from "@app/lib/utils";

  import Layout from "./Layout.svelte";

  import Icon from "@app/components/Icon.svelte";
  import Link from "@app/components/Link.svelte";
  import NodeId from "@app/components/NodeId.svelte";

  export let repo: RepoInfo;
  export let patches: Patch[];
  export let config: Config;
  export let status: PatchStatus;

  const statusColor: Record<Patch["state"]["status"], string> = {
    draft: "var(--color-fill-gray)",
    open: "var(--color-fill-success)",
    archived: "var(--color-foreground-yellow)",
    merged: "var(--color-fill-primary)",
  };

  const statusBackgroundColor: Record<Patch["state"]["status"], string> = {
    draft: "var(--color-fill-ghost)",
    open: "var(--color-fill-diff-green)",
    archived: "var(--color-fill-private)",
    merged: "var(--color-fill-delegate)",
  };

  $: project = repo.payloads["xyz.radicle.project"]!;
</script>

<style>
  .list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
</style>

<Layout {repo} selfDid={`did:key:${config.publicKey}`}>
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

  <svelte:fragment slot="sidebar">
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
        variant={status === "all" ? "active" : "tab"}
        route={{ resource: "repo.patches", rid: repo.rid, status: "all" }}>
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
    {#each patches as patch}
      <div class="global-flex">
        <div
          class="global-counter"
          style:padding="0"
          style:color={statusColor[patch.state.status]}
          style:background-color={statusBackgroundColor[patch.state.status]}>
          <Icon name="patch" />
        </div>
        <div class="global-oid">{formatOid(patch.id)}</div>
        {patch.title}
      </div>
    {/each}

    {#if patches.length === 0}
      {#if status === "all"}
        No patches.
      {:else}
        No {status} patches.
      {/if}
    {/if}
  </div>
</Layout>
