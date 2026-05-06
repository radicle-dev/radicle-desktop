<script lang="ts" module>
  import type { Author } from "@bindings/cob/Author";
  import type { Action } from "@bindings/cob/patch/Action";

  export type FlattenedPatchOperation = Action & {
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
    patchStatusColor,
    pluralize,
  } from "@app/lib/utils";

  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import NodeId from "@app/components/NodeId.svelte";

  interface Props {
    op: FlattenedPatchOperation;
    expanded?: boolean;
    onToggle?: () => void;
  }

  const { op, expanded, onToggle }: Props = $props();

  function lastLine(text: string): string | undefined {
    const lines = text.trim().split("\n");
    const last = lines[lines.length - 1]?.trim();
    return last && last.length > 0 ? last : undefined;
  }

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
  .timeline-item.toggleable {
    cursor: pointer;
  }
  .wrapper {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
    min-width: 0;
    flex: 1 1 0;
  }
  .icon {
    padding-top: 0.1875rem;
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
  .verdict-accept {
    background-color: var(--color-feedback-success-bg);
    color: var(--color-feedback-success-text);
    padding: 0.375rem 0.5rem;
    border-radius: var(--border-radius-sm);
  }
  .verdict-accept,
  .verdict-accept :global(*) {
    color: var(--color-feedback-success-text);
  }
  .verdict-reject {
    background-color: var(--color-feedback-error-bg);
    color: var(--color-feedback-error-text);
    padding: 0.375rem 0.5rem;
    border-radius: var(--border-radius-sm);
  }
  .verdict-reject,
  .verdict-reject :global(*) {
    color: var(--color-feedback-error-text);
  }
</style>

{#if op.type === "revision"}
  {@const summary = lastLine(op.description)}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <div
    class="timeline-item txt-body-m-regular"
    class:toggleable={onToggle !== undefined}
    role={onToggle ? "button" : undefined}
    tabindex={onToggle ? 0 : undefined}
    onclick={onToggle}>
    <div class="icon">
      <Icon
        name={onToggle
          ? expanded
            ? "chevron-down"
            : "chevron-right"
          : "revision"} />
    </div>
    <div class="wrapper">
      <NodeId {...authorForNodeId(op.author)} />
      <div class="summary-line">
        created revision <Id id={op.id} clipboard={op.id} />
        {#if summary}
          — {summary}
        {/if}
      </div>
      <div class="timestamp" title={absoluteTimestamp(op.timestamp)}>
        {formatTimestamp(op.timestamp)}
      </div>
    </div>
  </div>
{:else if op.type === "lifecycle"}
  <div class="timeline-item txt-body-m-regular">
    <div class="icon" style:color={patchStatusColor[op.state.status]}>
      <Icon
        name={op.state.status === "open"
          ? "patch"
          : `patch-${op.state.status}`} />
    </div>
    <div class="wrapper">
      <NodeId {...authorForNodeId(op.author)} />
      <div class="summary-line">
        {#if op.state.status === "draft"}
          converted patch to draft
        {:else if op.state.status === "archived"}
          archived patch
        {:else if op.state.status === "open"}
          reopened patch
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
      </div>
      <div class="timestamp" title={absoluteTimestamp(op.timestamp)}>
        {formatTimestamp(op.timestamp)}
      </div>
    </div>
  </div>
{:else if op.type === "merge"}
  <div class="timeline-item txt-body-m-regular">
    <div class="icon" style:color="var(--color-brand-bg)">
      <Icon name="patch-merged" />
    </div>
    <div class="wrapper">
      <NodeId {...authorForNodeId(op.author)} />
      <div class="summary-line">
        merged patch at revision <Id id={op.revision} clipboard={op.revision} />
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
{:else if op.type === "review"}
  {#if op.verdict === "accept"}
    <div class="timeline-item txt-body-m-regular verdict-accept">
      <div class="icon" style:color="var(--color-feedback-success-text)">
        <Icon name="thumbs-up" />
      </div>
      <div class="wrapper">
        <NodeId {...authorForNodeId(op.author)} />
        <div class="summary-line">
          accepted revision <Id id={op.revision} clipboard={op.revision} />
          {#if op.summary && op.summary.trim() !== ""}
            — {op.summary}
          {/if}
        </div>
        <div class="timestamp" title={absoluteTimestamp(op.timestamp)}>
          {formatTimestamp(op.timestamp)}
        </div>
      </div>
    </div>
  {:else if op.verdict === "reject"}
    <div class="timeline-item txt-body-m-regular verdict-reject">
      <div class="icon" style:color="var(--color-feedback-error-text)">
        <Icon name="stop" />
      </div>
      <div class="wrapper">
        <NodeId {...authorForNodeId(op.author)} />
        <div class="summary-line">
          rejected revision <Id id={op.revision} clipboard={op.revision} />
          {#if op.summary && op.summary.trim() !== ""}
            — {op.summary}
          {/if}
        </div>
        <div class="timestamp" title={absoluteTimestamp(op.timestamp)}>
          {formatTimestamp(op.timestamp)}
        </div>
      </div>
    </div>
  {:else if op.verdict === undefined}
    <div class="timeline-item txt-body-m-regular">
      <div class="icon">
        <Icon name="comment" />
      </div>
      <div class="wrapper">
        <NodeId {...authorForNodeId(op.author)} />
        <div class="summary-line">
          reviewed revision <Id id={op.revision} clipboard={op.revision} />
        </div>
        <div class="timestamp" title={absoluteTimestamp(op.timestamp)}>
          {formatTimestamp(op.timestamp)}
        </div>
      </div>
    </div>
  {/if}
{/if}
