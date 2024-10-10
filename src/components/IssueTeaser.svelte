<script lang="ts">
  import type { Issue } from "@bindings/Issue";

  import {
    formatOid,
    formatTimestamp,
    issueStatusBackgroundColor,
    issueStatusColor,
  } from "@app/lib/utils";
  import { push } from "@app/lib/router";

  import Icon from "./Icon.svelte";
  import InlineTitle from "./InlineTitle.svelte";
  import NodeId from "./NodeId.svelte";

  export let issue: Issue;
  export let rid: string;
</script>

<style>
  .issue-teaser {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.25rem;
    height: 5rem;
    background-color: var(--color-background-float);
    padding: 1rem;
    cursor: pointer;
    font-size: var(--font-size-regular);
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
  onclick={() => {
    void push({ resource: "repo.issue", rid, issue: issue.id });
  }}>
  <div class="global-flex">
    <div
      class="global-counter status"
      style:color={issueStatusColor[issue.state.status]}
      style:background-color={issueStatusBackgroundColor[issue.state.status]}>
      <Icon name="issue" />
    </div>
    <div
      class="global-flex"
      style:flex-direction="column"
      style:align-items="flex-start">
      <InlineTitle content={issue.title} />
      <div class="global-flex txt-small">
        <NodeId
          nodeId={issue.author.did.replace("did:key:", "")}
          alias={issue.author.alias} />
        opened
        <div class="global-oid">{formatOid(issue.id)}</div>
        {formatTimestamp(issue.timestamp)}
      </div>
    </div>
  </div>
  <div class="global-flex">
    {#each issue.labels as label}
      <div class="global-counter txt-small">{label}</div>
    {/each}

    {#if issue.discussion.length > 1}
      <div class="txt-small global-flex" style:gap="0.25rem">
        <Icon name="comment" />
        {issue.discussion.length - 1}
      </div>
    {/if}
  </div>
</div>
