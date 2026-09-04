<script lang="ts" module>
  import type { Author } from "@bindings/cob/Author";
  import type { Action } from "@bindings/cob/issue/Action";

  export type FlattenedIssueOperation =
    | (Action & {
        id: string;
        author: Author;
        timestamp: number;
        previous?: Action;
      })
    | { type: "opened"; id: string; author: Author; timestamp: number };
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
  import Label from "@app/components/Label.svelte";
  import NodeId from "@app/components/NodeId.svelte";

  interface Props {
    op: FlattenedIssueOperation;
    // Set when the surrounding run already names the author above the group.
    hideAuthor?: boolean;
  }

  const { op, hideAuthor }: Props = $props();

  function itemDiff<A>(previousState: A[], newState: A[]) {
    const removed = previousState.filter(x => !newState.includes(x));
    const added = newState.filter(x => !previousState.includes(x));
    return { removed, added };
  }
</script>

<style>
  /* One flat row: the 0.5rem left padding plus the 1rem icon place the icon's
     centre on Discussion's rail at 1rem (the rail paints an opaque background
     behind `.icon` so it threads behind the row). */
  .timeline-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
    padding: 0.375rem 0.5rem;
    min-height: 2.5rem;
  }
  .icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    width: 1rem;
  }
  .wrapper {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    min-width: 0;
    flex: 1 1 0;
  }
  /* Flex row so text and inline chips (labels, assignees) sit on one line. */
  .summary {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    flex: 1 1 0;
    min-width: 0;
    overflow: hidden;
    color: var(--color-text-primary);
  }
  /* Text portion truncates with an ellipsis; chips keep their size. Ellipsis
     has to sit on a child: it does not apply to the anonymous flex items a
     flex container makes of its text nodes. */
  .summary-text {
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .meta {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-shrink: 0;
  }
  .timestamp {
    color: var(--color-text-quaternary);
  }
</style>

{#snippet meta()}
  <div class="meta">
    <span class="timestamp" title={absoluteTimestamp(op.timestamp)}>
      {formatTimestamp(op.timestamp)}
    </span>
  </div>
{/snippet}

{#if op.type === "opened"}
  <div class="timeline-item txt-body-m-regular">
    <div class="icon" style:color={issueStatusColor["open"]}>
      <Icon name="issue" />
    </div>
    <div class="wrapper">
      {#if !hideAuthor}<NodeId {...authorForNodeId(op.author)} />{/if}
      <span class="summary">
        <span class="summary-text">opened this issue</span>
      </span>
      {@render meta()}
    </div>
  </div>
{:else if op.type === "lifecycle"}
  <div class="timeline-item txt-body-m-regular">
    <div class="icon" style:color={issueStatusColor[op.state.status]}>
      <Icon name={op.state.status === "open" ? "issue" : "issue-closed"} />
    </div>
    <div class="wrapper">
      {#if !hideAuthor}<NodeId {...authorForNodeId(op.author)} />{/if}
      <span class="summary">
        <span class="summary-text">
          {#if op.state.status === "closed"}
            closed this issue as {op.state.reason}
          {:else}
            reopened this issue
          {/if}
        </span>
      </span>
      {@render meta()}
    </div>
  </div>
{:else if op.type === "label"}
  <div class="timeline-item txt-body-m-regular">
    <div class="icon"><Icon name="label" /></div>
    <div class="wrapper">
      {#if !hideAuthor}<NodeId {...authorForNodeId(op.author)} />{/if}
      <span class="summary">
        {#if op.previous && op.previous.type === op.type}
          {@const changed = itemDiff(op.previous?.labels ?? [], op.labels)}
          {#if changed.added.length}
            <span class="summary-text">
              added {pluralize("label", changed.added.length)}
            </span>
            {#each changed.added as label}<Label {label} />{/each}
          {/if}
          {#if changed.removed.length}
            <span class="summary-text">
              removed {pluralize("label", changed.removed.length)}
            </span>
            {#each changed.removed as label}<Label {label} />{/each}
          {/if}
        {:else}
          <span class="summary-text">
            added {pluralize("label", op.labels.length)}
          </span>
          {#each op.labels as label}<Label {label} />{/each}
        {/if}
      </span>
      {@render meta()}
    </div>
  </div>
{:else if op.type === "assign"}
  <div class="timeline-item txt-body-m-regular">
    <div class="icon"><Icon name="avatar-incognito" /></div>
    <div class="wrapper">
      {#if !hideAuthor}<NodeId {...authorForNodeId(op.author)} />{/if}
      <span class="summary">
        {#if op.previous && op.previous.type === op.type}
          {@const changed = itemDiff<Author>(
            op.previous?.assignees ?? [],
            op.assignees,
          )}
          {#if changed.added.length}
            <span class="summary-text">assigned</span>
            {#each changed.added as assignee}
              <NodeId {...authorForNodeId(assignee)} />
            {/each}
          {/if}
          {#if changed.removed.length}
            <span class="summary-text">unassigned</span>
            {#each changed.removed as assignee}
              <NodeId {...authorForNodeId(assignee)} />
            {/each}
          {/if}
        {:else}
          <span class="summary-text">assigned</span>
          {#each op.assignees as assignee}
            <NodeId {...authorForNodeId(assignee)} />
          {/each}
        {/if}
      </span>
      {@render meta()}
    </div>
  </div>
{:else if op.type === "edit"}
  {#if op.previous && op.previous.type === op.type}
    <div class="timeline-item txt-body-m-regular">
      <div class="icon"><Icon name="edit" /></div>
      <div class="wrapper">
        {#if !hideAuthor}<NodeId {...authorForNodeId(op.author)} />{/if}
        <span class="summary">
          <span
            class="summary-text"
            title={`changed the title from “${op.previous.title}” to “${op.title}”`}>
            changed the title from “{op.previous.title}” to “{op.title}”
          </span>
        </span>
        {@render meta()}
      </div>
    </div>
  {/if}
{/if}
