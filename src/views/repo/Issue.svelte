<script lang="ts">
  import type { Config } from "@bindings/Config";
  import type { Issue } from "@bindings/Issue";
  import type { RepoInfo } from "@bindings/RepoInfo";

  import capitalize from "lodash/capitalize";

  import { formatTimestamp, formatOid, issueStatusColor } from "@app/lib/utils";

  import Border from "@app/components/Border.svelte";
  import CopyableId from "@app/components/CopyableId.svelte";
  import Icon from "@app/components/Icon.svelte";
  import InlineTitle from "@app/components/InlineTitle.svelte";
  import Layout from "./Layout.svelte";
  import Link from "@app/components/Link.svelte";
  import Markdown from "@app/components/Markdown.svelte";
  import NodeId from "@app/components/NodeId.svelte";

  export let repo: RepoInfo;
  export let issue: Issue;
  export let issues: Issue[];
  export let config: Config;

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
  .issue-teaser {
    max-width: 11rem;
    white-space: nowrap;
  }
  .issue-list {
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
    margin-top: 1rem;
    clip-path: var(--2px-corner-fill);
  }
  .divider {
    width: 2px;
    background-color: var(--color-fill-ghost);
    height: calc(100% + 8px);
    top: -2px;
    position: relative;
  }
  .section {
    padding: 0.5rem;
    font-size: var(--font-size-small);
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    height: 100%;
  }
  .section-title {
    margin-bottom: 0.5rem;
    color: var(--color-foreground-dim);
  }
</style>

<Layout>
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

  <svelte:fragment slot="header-center">
    <CopyableId id={issue.id} />
  </svelte:fragment>

  <svelte:fragment slot="sidebar">
    <Border
      hoverable={false}
      variant="ghost"
      styleWidth="100%"
      styleHeight="32px">
      <div style:margin-left="0.5rem">
        <Icon name="issue" />
      </div>
      <span class="txt-small txt-semibold">Issues</span>
      <div class="global-flex txt-small" style:margin-left="auto">
        <div
          class="global-counter"
          style:padding="0 6px"
          style:background-color="var(--color-fill-ghost)"
          style:gap="4px">
          {project.meta.issues.open + project.meta.issues.closed}
        </div>
      </div>
    </Border>

    <div class="issue-list">
      {#each issues as sidebarIssue}
        <Link
          variant="tab"
          route={{
            resource: "repo.issue",
            rid: repo.rid,
            issue: sidebarIssue.id,
          }}>
          <div class="global-flex">
            <div
              style:color={issueStatusColor[sidebarIssue.state.status]}
              style:margin-left="2px">
              <Icon name="issue" />
            </div>
            <span class="txt-small issue-teaser txt-overflow">
              <InlineTitle content={sidebarIssue.title} fontSize="small" />
            </span>
          </div>
        </Link>
      {/each}
    </div>
  </svelte:fragment>

  <div class="content">
    <div class="title">
      <InlineTitle content={issue.title} fontSize="medium" />
    </div>

    <Border variant="ghost" styleGap="0">
      <div class="section" style:min-width="8rem">
        <div class="section-title">Status</div>
        <div
          class="global-counter txt-small"
          style:width="fit-content"
          style:color="var(--color-foreground-match-background)"
          style:background-color={issueStatusColor[issue.state.status]}>
          {capitalize(issue.state.status)}
        </div>
      </div>

      <div class="divider"></div>

      <div class="section" style:flex="1">
        <div class="section-title">Labels</div>
        <div class="global-flex" style:flex-wrap="wrap">
          {#each issue.labels as label}
            <div class="global-counter txt-small">{label}</div>
          {:else}
            <span class="txt-missing">No labels.</span>
          {/each}
        </div>
      </div>

      <div class="divider"></div>

      <div class="section" style:flex="1">
        <div class="section-title">Assignees</div>
        <div class="global-flex" style:flex-wrap="wrap">
          {#each issue.assignees as assignee}
            <NodeId
              nodeId={assignee.did.replace("did:key:", "")}
              alias={assignee.alias} />
          {:else}
            <span class="txt-missing">Not assigned to anyone.</span>
          {/each}
        </div>
      </div>
    </Border>

    <div class="txt-small body">
      {#if issue.discussion[0].edits.slice(-1)[0].body.trim() === ""}
        <span class="txt-missing">No description.</span>
      {:else}
        <Markdown
          rid={repo.rid}
          breaks
          content={issue.discussion[0].edits.slice(-1)[0].body} />
      {/if}
      <div class="global-flex txt-small" style:margin-top="1.5rem">
        <NodeId
          nodeId={issue.author.did.replace("did:key:", "")}
          alias={issue.author.alias} />
        opened
        <div class="global-oid">{formatOid(issue.id)}</div>
        {formatTimestamp(issue.timestamp)}
      </div>
    </div>
  </div>
</Layout>
