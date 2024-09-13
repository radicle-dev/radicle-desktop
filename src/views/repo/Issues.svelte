<script lang="ts">
  import type { Config } from "@bindings/Config";
  import type { Issue } from "@bindings/Issue";
  import type { IssueStatus } from "./router";
  import type { RepoInfo } from "@bindings/RepoInfo";

  import { formatOid } from "@app/lib/utils";

  import Layout from "./Layout.svelte";

  import Icon from "@app/components/Icon.svelte";
  import Link from "@app/components/Link.svelte";
  import NodeId from "@app/components/NodeId.svelte";

  export let repo: RepoInfo;
  export let issues: Issue[];
  export let config: Config;
  export let status: IssueStatus;

  const statusColor: Record<Issue["state"]["status"], string> = {
    open: "var(--color-fill-success)",
    closed: "var(--color-foreground-red)",
  };
  const statusBackgroundColor: Record<Issue["state"]["status"], string> = {
    open: "var(--color-fill-diff-green)",
    closed: "var(--color-fill-diff-red)",
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
    Issues
  </svelte:fragment>

  <svelte:fragment slot="sidebar">
    <div class="global-flex txt-small" style:margin="0.5rem 0">
      <Link
        variant={status === "all" ? "active" : "tab"}
        route={{ resource: "repo.issues", rid: repo.rid, status: "all" }}>
        <div class="global-flex"><Icon name="issue" />Issues</div>
        <div class="global-counter">
          {project.meta.issues.open + project.meta.issues.closed}
        </div>
      </Link>
    </div>
    <div class="global-flex txt-small global-tab">
      <Link
        variant={status === "open" ? "active" : "tab"}
        route={{ resource: "repo.issues", rid: repo.rid, status: "open" }}>
        Open
        <div class="global-counter">
          {project.meta.issues.open}
        </div>
      </Link>
      <Link
        variant={status === "closed" ? "active" : "tab"}
        route={{
          resource: "repo.issues",
          rid: repo.rid,
          status: "closed",
        }}>
        Closed
        <div class="global-counter">
          {project.meta.issues.closed}
        </div>
      </Link>
    </div>

    <div class="global-flex txt-small" style:margin="0.5rem 0">
      <Link
        variant="tab"
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
  </svelte:fragment>

  <div class="list">
    {#each issues as issue}
      <div class="global-flex">
        <div
          class="global-counter"
          style:padding="0"
          style:color={statusColor[issue.state.status]}
          style:background-color={statusBackgroundColor[issue.state.status]}>
          <Icon name="issue" />
        </div>
        <div class="global-oid">{formatOid(issue.id)}</div>
        {issue.title}
      </div>
    {/each}

    {#if issues.length === 0}
      {#if status === "all"}
        No issues.
      {:else}
        No {status} issues.
      {/if}
    {/if}
  </div>
</Layout>
