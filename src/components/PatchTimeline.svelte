<script lang="ts">
  import type { Action } from "@bindings/cob/patch/Action";
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
    patchStatusColor,
    pluralize,
  } from "@app/lib/utils";
  import Icon from "./Icon.svelte";
  import Id from "./Id.svelte";
  import NodeId from "./NodeId.svelte";

  interface Props {
    patchId: string;
    activity: Operation<Action>[];
  }

  /* eslint-disable prefer-const */
  let { activity, patchId }: Props = $props();
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
    padding-top: 0.1875rem;
  }
</style>

<div class="timeline txt-small">
  {#each timeline as op}
    {#if op.type === "revision"}
      {#if op.id === patchId}
        <div class="timeline-item">
          <div class="icon" style:color="var(--color-fill-success)">
            <Icon name="patch" />
          </div>
          <div class="wrapper">
            <NodeId {...authorForNodeId(op.author)} />
            <div>opened patch <Id id={op.id} variant="oid" /></div>
            <div title={absoluteTimestamp(op.timestamp)}>
              {formatTimestamp(op.timestamp)}
            </div>
          </div>
        </div>
      {:else}
        <div class="timeline-item">
          <div class="icon">
            <Icon name="revision" />
          </div>
          <div class="wrapper">
            <NodeId {...authorForNodeId(op.author)} />
            <div>created revision <Id id={op.id} variant="oid" /></div>
            <div title={absoluteTimestamp(op.timestamp)}>
              {formatTimestamp(op.timestamp)}
            </div>
          </div>
        </div>
      {/if}
    {:else if op.type === "lifecycle"}
      <div class="timeline-item">
        <div class="icon" style:color={patchStatusColor[op.state.status]}>
          <Icon
            name={op.state.status === "open"
              ? "patch"
              : `patch-${op.state.status}`} />
        </div>
        <div class="wrapper">
          <NodeId {...authorForNodeId(op.author)} />
          {#if op.state.status === "draft"}
            converted patch to draft
          {:else if op.state.status === "archived"}
            archived patch
          {:else if op.state.status === "open"}
            reopened patch
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
                added {pluralize("label", changed.added.length)}
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
    {:else if op.type === "merge"}
      <div class="timeline-item">
        <div class="icon" style:color="var(--color-fill-primary)">
          <Icon name="patch-merged" />
        </div>
        <div class="wrapper">
          <NodeId {...authorForNodeId(op.author)} />
          <div>
            merged patch at revision <Id id={op.revision} variant="oid" />
          </div>
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
    {:else if op.type === "review"}
      <div class="timeline-item">
        {#if op.verdict === "accept"}
          <div class="icon" style:color="var(--color-foreground-success)">
            <Icon name="comment-checkmark" />
          </div>
          <div class="wrapper">
            <NodeId {...authorForNodeId(op.author)} />
            accepted revision <Id id={op.revision} variant="oid" />
            <div title={absoluteTimestamp(op.timestamp)}>
              {formatTimestamp(op.timestamp)}
            </div>
          </div>
        {:else if op.verdict === "reject"}
          <div class="icon" style:color="var(--color-foreground-red)">
            <Icon name="comment-cross" />
          </div>
          <div class="wrapper">
            <NodeId {...authorForNodeId(op.author)} />
            rejected revision <Id id={op.revision} variant="oid" />
            <div title={absoluteTimestamp(op.timestamp)}>
              {formatTimestamp(op.timestamp)}
            </div>
          </div>
        {:else if op.verdict === undefined}
          <div class="icon">
            <Icon name="comment" />
          </div>
          <div class="wrapper">
            <NodeId {...authorForNodeId(op.author)} />
            reviewed revision <Id id={op.revision} variant="oid" />
            <div title={absoluteTimestamp(op.timestamp)}>
              {formatTimestamp(op.timestamp)}
            </div>
          </div>
        {/if}
      </div>
    {:else if op.type === "review.comment"}
      <div class="timeline-item">
        <div class="icon">
          <Icon name="comment" />
        </div>
        <div class="wrapper">
          <NodeId {...authorForNodeId(op.author)} />
          {op.replyTo ? "replied to a comment" : "commented"} on review <Id
            id={op.review}
            variant="oid" />
          <div title={absoluteTimestamp(op.timestamp)}>
            {formatTimestamp(op.timestamp)}
          </div>
        </div>
      </div>
    {:else if op.type === "revision.comment"}
      <div class="timeline-item">
        <div class="icon">
          <Icon name="comment" />
        </div>
        <div class="wrapper">
          <NodeId {...authorForNodeId(op.author)} />
          {op.replyTo ? "replied to a comment" : "commented"} on revision <Id
            id={op.revision}
            variant="oid" />
          <div title={absoluteTimestamp(op.timestamp)}>
            {formatTimestamp(op.timestamp)}
          </div>
        </div>
      </div>
    {/if}
  {/each}
</div>
