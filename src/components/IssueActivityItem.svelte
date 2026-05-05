<script lang="ts" module>
  import type { Author } from "@bindings/cob/Author";
  import type { Action } from "@bindings/cob/issue/Action";

  export type FlattenedIssueOperation = Action & {
    id: string;
    author: Author;
    timestamp: number;
    previous?: Action;
  };
</script>

<script lang="ts">
  import {
    absoluteTimestamp,
    authorForNodeId,
    formatTimestamp,
    issueStatusColor,
    pluralize,
  } from "@app/lib/utils";

  import Icon from "@app/components/Icon.svelte";
  import NodeId from "@app/components/NodeId.svelte";

  interface Props {
    op: FlattenedIssueOperation;
  }

  const { op }: Props = $props();

  function itemDiff<A>(previousState: A[], newState: A[]) {
    const removed = previousState.filter(x => !newState.includes(x));
    const added = newState.filter(x => !previousState.includes(x));
    return { removed, added };
  }
</script>

<style>
  .timeline-item {
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
    min-width: 0;
  }
  .wrapper {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
    min-width: 0;
    flex: 1 1 0;
  }
  .icon {
    padding-top: 3px;
  }
  .summary-line {
    flex: 1 1 0;
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .timestamp {
    color: var(--color-text-quaternary);
  }
</style>

{#if op.type === "lifecycle"}
  <div class="timeline-item txt-body-m-regular">
    <div class="icon" style:color={issueStatusColor[op.state.status]}>
      {#if op.state.status === "open"}
        <Icon name="issue" />
      {:else}
        <Icon name="issue-closed" />
      {/if}
    </div>
    <div class="wrapper">
      <NodeId {...authorForNodeId(op.author)} />
      <div class="summary-line">
        {#if op.state.status === "closed"}
          closed issue as {op.state.reason}
        {:else if op.state.status === "open"}
          reopened issue
        {/if}
      </div>
      <div class="timestamp" title={absoluteTimestamp(op.timestamp)}>
        {formatTimestamp(op.timestamp)}
      </div>
    </div>
  </div>
{:else if op.type === "label"}
  <div class="timeline-item txt-body-m-regular">
    <div class="icon">
      <Icon name="label" />
    </div>
    <div class="wrapper">
      <NodeId {...authorForNodeId(op.author)} />
      <div class="summary-line">
        {#if op.previous && op.previous.type === op.type}
          {@const changed = itemDiff(op.previous?.labels ?? [], op.labels)}
          {#if changed.added.length || changed.removed.length}
            {#if changed.added.length}
              added {pluralize("label", changed.removed.length)}
              {#each changed.added as label}
                <b>{label}</b>
              {/each}
            {/if}
            {#if changed.removed.length}
              removed {pluralize("label", changed.removed.length)}
              {#each changed.removed as label}
                <b>{label}</b>
              {/each}
            {/if}
          {/if}
        {:else}
          added {pluralize("label", op.labels.length)}
          {#each op.labels as label}
            <b>{label}</b>
          {/each}
        {/if}
      </div>
      <div class="timestamp" title={absoluteTimestamp(op.timestamp)}>
        {formatTimestamp(op.timestamp)}
      </div>
    </div>
  </div>
{:else if op.type === "assign"}
  <div class="timeline-item txt-body-m-regular">
    <div class="icon">
      <Icon name="avatar-incognito" />
    </div>
    <div class="wrapper">
      <NodeId {...authorForNodeId(op.author)} />
      <div class="summary-line">
        {#if op.previous && op.previous.type === op.type}
          {@const changed = itemDiff<Author>(
            op.previous?.assignees ?? [],
            op.assignees,
          )}
          {#if changed.added.length || changed.removed.length}
            {#if changed.added.length}
              assigned
              {#each changed.added as assignee}
                <NodeId {...authorForNodeId(assignee)} />
              {/each}
            {/if}
            {#if changed.removed.length}
              unassigned
              {#each changed.removed as assignee}
                <NodeId {...authorForNodeId(assignee)} />
              {/each}
            {/if}
          {/if}
        {:else}
          assigned
          {#each op.assignees as assignee}
            <NodeId {...authorForNodeId(assignee)} />
          {/each}
        {/if}
      </div>
      <div class="timestamp" title={absoluteTimestamp(op.timestamp)}>
        {formatTimestamp(op.timestamp)}
      </div>
    </div>
  </div>
{:else if op.type === "edit"}
  {#if op.previous && op.previous.type === op.type}
    <div class="timeline-item txt-body-m-regular">
      <div class="icon">
        <Icon name="edit" />
      </div>
      <div class="wrapper">
        <NodeId {...authorForNodeId(op.author)} />
        <div class="summary-line">
          changed title <s>{op.previous.title}</s>
          → {op.title}
        </div>
        <div class="timestamp" title={absoluteTimestamp(op.timestamp)}>
          {formatTimestamp(op.timestamp)}
        </div>
      </div>
    </div>
  {/if}
{/if}
