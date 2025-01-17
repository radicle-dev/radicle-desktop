<script lang="ts">
  import type { Issue } from "@bindings/cob/issue/Issue";
  import type { IssueStatus } from "@app/views/repo/router";

  import {
    authorForNodeId,
    formatTimestamp,
    issueStatusBackgroundColor,
    issueStatusColor,
  } from "@app/lib/utils";
  import { push } from "@app/lib/router";

  import Icon from "./Icon.svelte";
  import Id from "./Id.svelte";
  import InlineTitle from "./InlineTitle.svelte";
  import Label from "./Label.svelte";
  import NodeId from "./NodeId.svelte";

  interface Props {
    issue: Issue;
    rid: string;
    status: IssueStatus;
    selected?: boolean;
    compact?: boolean;
  }

  const {
    issue,
    rid,
    status,
    selected = false,
    compact = false,
  }: Props = $props();
</script>

<style>
  .issue-teaser {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.25rem;
    min-height: 5rem;
    background-color: var(--color-background-float);
    padding: 1rem;
    cursor: pointer;
    font-size: var(--font-size-regular);
    word-break: break-word;
  }
  .selected {
    background-color: var(--color-fill-float-hover);
  }
  .issue-teaser:hover {
    background-color: var(--color-fill-float-hover);
  }
  .status {
    padding: 0;
    margin-right: 1rem;
  }
  .issue-teaser:first-of-type {
    clip-path: var(--3px-top-corner-fill);
  }
  .issue-teaser:last-of-type {
    clip-path: var(--3px-bottom-corner-fill);
  }
  .issue-teaser:only-of-type {
    clip-path: var(--3px-corner-fill);
  }
</style>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  tabindex="0"
  role="button"
  class="issue-teaser"
  class:selected
  onclick={() => {
    void push({ resource: "repo.issue", rid, issue: issue.id, status });
  }}>
  <div class="global-flex">
    <div
      class="global-counter status"
      style:color={issueStatusColor[issue.state.status]}
      style:background-color={issueStatusBackgroundColor[issue.state.status]}>
      {#if issue.state.status === "open"}
        <Icon name="issue" />
      {:else}
        <Icon name="issue-closed" />
      {/if}
    </div>
    <div
      class="global-flex"
      style:flex-direction="column"
      style:align-items="flex-start">
      <InlineTitle content={issue.title} />
      <div class="global-flex txt-small" style:flex-wrap="wrap">
        <NodeId {...authorForNodeId(issue.author)} />
        opened
        <Id id={issue.id} variant="oid" />
        {formatTimestamp(issue.timestamp)}
      </div>
    </div>
  </div>

  <div class="global-flex">
    {#if !compact}
      {#each issue.labels as label}
        <Label {label} />
      {/each}
    {/if}

    {#if issue.commentCount > 0}
      <div class="txt-small global-flex" style:gap="0.25rem">
        <Icon name="comment" />
        {issue.commentCount}
      </div>
    {/if}
  </div>
</div>
