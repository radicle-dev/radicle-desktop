<script lang="ts">
  import type { Config } from "@bindings/config/Config";
  import type { Issue } from "@bindings/cob/issue/Issue";
  import type { IssueStatus } from "./router";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import * as router from "@app/lib/router";

  import Layout from "./Layout.svelte";

  import Button from "@app/components/Button.svelte";
  import CopyableId from "@app/components/CopyableId.svelte";
  import Icon from "@app/components/Icon.svelte";
  import IssueTeaser from "@app/components/IssueTeaser.svelte";
  import IssuesSecondColumn from "@app/components/IssuesSecondColumn.svelte";
  import Link from "@app/components/Link.svelte";
  import NodeId from "@app/components/NodeId.svelte";
  import Sidebar from "@app/components/Sidebar.svelte";

  interface Props {
    repo: RepoInfo;
    issues: Issue[];
    config: Config;
    status: IssueStatus;
  }

  const { repo, issues, config, status }: Props = $props();

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
    display: flex;
    padding: 1rem 1rem 0.5rem 1rem;
    align-items: center;
    justify-content: space-between;
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
    padding-left: 1rem;
    color: var(--color-foreground-dim);
  }
</style>

<Layout
  hideSidebar
  styleSecondColumnOverflow="visible"
  publicKey={config.publicKey}>
  {#snippet headerCenter()}
    <CopyableId id={repo.rid} />
  {/snippet}

  {#snippet sidebar()}
    <Sidebar activeTab={{ type: "issues", status }} rid={repo.rid} />
  {/snippet}

  {#snippet secondColumn()}
    <div style:margin-left="1rem" style:height="100%">
      <IssuesSecondColumn {project} {status} {repo} />
    </div>
  {/snippet}

  <div class="header">
    <div>Issues</div>
    <div class="txt-regular txt-semibold">
      <Button
        variant="secondary"
        onclick={() => {
          void router.push({
            resource: "repo.createIssue",
            status,
            rid: repo.rid,
          });
        }}>
        <Icon name="plus" />New
      </Button>
    </div>
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
      route={{ resource: "repo.issues", rid: repo.rid, status: "open" }}
      styleColor="var(--color-foreground-dim)">
      {project.data.name}
    </Link>
    <Icon name="chevron-right" />
    Issues
  </div>

  <div class="list">
    {#each issues as issue}
      <IssueTeaser {issue} rid={repo.rid} {status} />
    {/each}

    {#if issues.length === 0}
      <div class="txt-missing txt-small" style:margin-left="1rem">
        {#if status === "all"}
          No issues.
        {:else}
          No {status} issues.
        {/if}
      </div>
    {/if}
  </div>
</Layout>
