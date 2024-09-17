<script lang="ts">
  import type { Issue } from "@bindings/Issue";

  import { formatOid } from "@app/lib/utils";

  import Icon from "./Icon.svelte";
  import NodeId from "./NodeId.svelte";

  export let issue: Issue;

  const statusColor: Record<Issue["state"]["status"], string> = {
    open: "var(--color-fill-success)",
    closed: "var(--color-foreground-red)",
  };
  const statusBackgroundColor: Record<Issue["state"]["status"], string> = {
    open: "var(--color-fill-diff-green)",
    closed: "var(--color-fill-diff-red)",
  };
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

<div class="issue-teaser">
  <div class="global-flex">
    <div
      class="global-counter status"
      style:color={statusColor[issue.state.status]}
      style:background-color={statusBackgroundColor[issue.state.status]}>
      <Icon name="issue" />
    </div>
    <div
      class="global-flex"
      style:flex-direction="column"
      style:align-items="flex-start">
      {issue.title}
      <div class="global-flex txt-small">
        <NodeId
          nodeId={issue.author.did.replace("did:key:", "")}
          alias={issue.author.alias} />
        opened
        <div class="global-oid">{formatOid(issue.id)}</div>
      </div>
    </div>
  </div>
  <div class="global-flex">
    {#each issue.labels as label}
      <div class="global-counter txt-small">{label}</div>
    {/each}
  </div>
</div>
