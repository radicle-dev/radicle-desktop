<script lang="ts">
  import type { Action } from "@bindings/cob/issue/Action";
  import type { Operation } from "@bindings/cob/Operation";
  import type { Author } from "@bindings/cob/Author";

  type FlattenedOperation = Action & {
    id: string;
    author: Author;
    timestamp: number;
    previous?: Action;
  };

  type StateTracker = Record<Action["type"], Action>;

  import {
    absoluteTimestamp,
    authorForNodeId,
    formatTimestamp,
    issueStatusColor,
    pluralize,
  } from "@app/lib/utils";
  import Icon from "./Icon.svelte";
  import NodeId from "./NodeId.svelte";
  import Id from "./Id.svelte";

  interface Props {
    activity: Operation<Action>[];
  }

  /* eslint-disable prefer-const */
  let { activity }: Props = $props();
  /* eslint-enable prefer-const */

  const timeline = $derived(enrichActivity(flattenActivity(activity)));

  function flattenActivity(
    activity: Operation<Action>[],
  ): FlattenedOperation[] {
    return activity.flatMap(operation =>
      operation.actions.map(action => ({
        ...action,
        id: operation.id,
        author: operation.author,
        timestamp: operation.timestamp,
      })),
    );
  }

  function enrichActivity(
    flatActivity: FlattenedOperation[],
  ): FlattenedOperation[] {
    const result: FlattenedOperation[] = [];
    const timelineStateTracker: StateTracker = {} as StateTracker;

    flatActivity.forEach(entry => {
      if (timelineStateTracker[entry.type]) {
        result.push({ ...entry, previous: timelineStateTracker[entry.type] });
      } else {
        result.push(entry);
      }
      timelineStateTracker[entry.type] = entry;
    });

    return result;
  }

  function itemDiff<A>(previousState: A[], newState: A[]) {
    const removed = previousState.filter(x => !newState.includes(x));
    const added = newState.filter(x => !previousState.includes(x));
    return { removed, added };
  }
</script>

<style>
  a {
    color: var(--color-foreground-default);
    text-decoration: none;
  }
  a:hover {
    text-decoration: underline;
    text-decoration-thickness: 1px;
    text-underline-offset: 2px;
  }
  .timeline {
    display: flex;
    gap: 0.75rem;
    flex-direction: column;
  }
  .timeline-item {
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
    word-break: break-word;
  }
  .wrapper {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
  }
  .icon {
    padding-top: 3px;
  }
</style>

<div class="timeline txt-small">
  {#each timeline as op}
    {#if op.type === "lifecycle"}
      <div class="timeline-item">
        <div class="icon" style:color={issueStatusColor[op.state.status]}>
          {#if op.state.status === "open"}
            <Icon name="issue" />
          {:else}
            <Icon name="issue-closed" />
          {/if}
        </div>
        <div class="wrapper">
          <NodeId {...authorForNodeId(op.author)} />
          {#if op.state.status === "closed"}
            closed issue as {op.state.reason}
          {:else if op.state.status === "open"}
            reopened issue
          {/if}
          <div title={absoluteTimestamp(op.timestamp)}>
            {formatTimestamp(op.timestamp)}
          </div>
        </div>
      </div>
    {:else if op.type === "label"}
      <div class="timeline-item">
        <div class="icon">
          <Icon name="label" />
        </div>
        <div class="wrapper">
          <NodeId {...authorForNodeId(op.author)} />
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
          <div title={absoluteTimestamp(op.timestamp)}>
            {formatTimestamp(op.timestamp)}
          </div>
        </div>
      </div>
    {:else if op.type === "assign"}
      <div class="timeline-item">
        <div class="icon">
          <Icon name="user" />
        </div>
        <div class="wrapper">
          <NodeId {...authorForNodeId(op.author)} />
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
          <div title={absoluteTimestamp(op.timestamp)}>
            {formatTimestamp(op.timestamp)}
          </div>
        </div>
      </div>
    {:else if op.type === "edit"}
      {#if op.previous && op.previous.type === op.type}
        <div class="timeline-item">
          <div class="icon">
            <Icon name="pen" />
          </div>
          <div class="wrapper">
            <NodeId {...authorForNodeId(op.author)} />
            changed title
            <s>{op.previous.title}</s>
            -> {op.title}
            <div title={absoluteTimestamp(op.timestamp)}>
              {formatTimestamp(op.timestamp)}
            </div>
          </div>
        </div>
      {/if}
    {:else if op.type === "comment"}
      {#if op.id === activity[0].id}
        <div class="timeline-item">
          <div class="icon" style:color="var(--color-fill-success)">
            <Icon name="issue" />
          </div>
          <div class="wrapper">
            <NodeId {...authorForNodeId(op.author)} />
            <div>opened issue <Id id={op.id} variant="oid" /></div>
            <div title={absoluteTimestamp(op.timestamp)}>
              {formatTimestamp(op.timestamp)}
            </div>
          </div>
        </div>
      {:else}
        <div class="timeline-item">
          <div class="icon">
            <Icon name="comment" />
          </div>
          <div class="wrapper">
            <NodeId {...authorForNodeId(op.author)} />
            <a href="#{op.id}">
              {op.replyTo && op.replyTo !== activity[0].id
                ? "replied to a comment"
                : "commented"}
            </a>
            <div title={absoluteTimestamp(op.timestamp)}>
              {formatTimestamp(op.timestamp)}
            </div>
          </div>
        </div>
      {/if}
    {/if}
  {/each}
</div>
