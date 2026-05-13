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

  import { slide } from "svelte/transition";

  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import NodeId from "@app/components/NodeId.svelte";

  interface Props {
    op: FlattenedPatchOperation;
    expanded?: boolean;
    onToggle?: () => void;
    hideAuthor?: boolean;
    targetBranch?: string;
  }

  const { op, expanded, onToggle, hideAuthor, targetBranch }: Props = $props();

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
    padding: 0.125rem 0.25rem;
    margin: 0 -0.25rem;
    border-radius: var(--border-radius-sm);
  }
  .timeline-item.toggleable:hover,
  .timeline-item.toggleable:focus-visible {
    background-color: var(--color-surface-subtle);
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
  .icon-stack {
    display: grid;
    width: 1rem;
    place-items: center;
  }
  .icon-default,
  .icon-hover {
    grid-area: 1 / 1;
    transition:
      opacity 150ms ease,
      transform 150ms ease;
  }
  .icon-hover {
    opacity: 0;
    transform: rotate(-90deg);
  }
  .timeline-item.toggleable:hover .icon-default,
  .timeline-item.toggleable:focus-visible .icon-default,
  .timeline-item.verdict-toggleable:hover .icon-default,
  .timeline-item.verdict-toggleable:focus-visible .icon-default {
    opacity: 0;
    transform: rotate(90deg);
  }
  .timeline-item.toggleable:hover .icon-hover,
  .timeline-item.toggleable:focus-visible .icon-hover,
  .timeline-item.verdict-toggleable:hover .icon-hover,
  .timeline-item.verdict-toggleable:focus-visible .icon-hover {
    opacity: 1;
    transform: rotate(0);
  }
  .verdict-toggleable {
    cursor: pointer;
  }
  .summary-line {
    flex: 1 1 0;
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .summary-secondary {
    color: var(--color-text-tertiary);
  }
  .summary-content {
    color: var(--color-text-primary);
  }
  .timestamp {
    color: var(--color-text-quaternary);
  }
  .meta {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-shrink: 0;
  }
  .meta-hash {
    color: var(--color-text-quaternary);
  }
  .meta-hash :global(.txt-id) {
    color: inherit;
  }
  .meta-hash :global(.txt-id:hover) {
    color: inherit;
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
  .verdict-summary {
    flex-basis: 100%;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  .merge-badge {
    background-color: var(--color-surface-brand-subtle);
    color: var(--color-text-brand);
    padding: 0.375rem 0.5rem;
    border-radius: var(--border-radius-sm);
  }
  .merge-badge,
  .merge-badge :global(*) {
    color: inherit;
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
      {#if onToggle}
        <span class="icon-stack">
          <span class="icon-default"><Icon name="revision" /></span>
          <span class="icon-hover">
            <Icon name={expanded ? "collapse-vertical" : "expand-vertical"} />
          </span>
        </span>
      {:else}
        <Icon name="revision" />
      {/if}
    </div>
    <div class="wrapper">
      {#if !hideAuthor}<NodeId {...authorForNodeId(op.author)} />{/if}
      <div class="summary-line">
        <span class="summary-secondary">created revision</span>
        {#if summary}
          <span class="txt-body-m-medium summary-content">{summary}</span>
        {/if}
      </div>
      <div class="meta">
        <div class="meta-hash">
          <Id id={op.id} clipboard={op.id} />
        </div>
        <div class="timestamp" title={absoluteTimestamp(op.timestamp)}>
          {formatTimestamp(op.timestamp)}
        </div>
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
      {#if !hideAuthor}<NodeId {...authorForNodeId(op.author)} />{/if}
      <div class="summary-line">
        <span class="summary-secondary">
          {#if op.state.status === "draft"}
            converted patch to draft
          {:else if op.state.status === "archived"}
            archived patch
          {:else if op.state.status === "open"}
            reopened patch
          {/if}
        </span>
      </div>
      <div class="meta">
        <div class="timestamp" title={absoluteTimestamp(op.timestamp)}>
          {formatTimestamp(op.timestamp)}
        </div>
      </div>
    </div>
  </div>
{:else if op.type === "label"}
  <div class="timeline-item txt-body-m-regular">
    <div class="icon">
      <Icon name="label" />
    </div>
    <div class="wrapper">
      {#if !hideAuthor}<NodeId {...authorForNodeId(op.author)} />{/if}
      <div class="summary-line">
        {#if op.previous && op.previous.type === op.type}
          {@const changed = itemDiff(op.previous?.labels ?? [], op.labels)}
          {#if changed.added.length || changed.removed.length}
            {#if changed.added.length}
              <span class="summary-secondary">
                added {pluralize("label", changed.added.length)}
              </span>
              {#each changed.added as label}
                <b class="summary-content">{label}</b>
              {/each}
            {/if}
            {#if changed.removed.length}
              <span class="summary-secondary">
                removed {pluralize("label", changed.removed.length)}
              </span>
              {#each changed.removed as label}
                <b class="summary-content">{label}</b>
              {/each}
            {/if}
          {/if}
        {:else}
          <span class="summary-secondary">
            added {pluralize("label", op.labels.length)}
          </span>
          {#each op.labels as label}
            <b class="summary-content">{label}</b>
          {/each}
        {/if}
      </div>
      <div class="meta">
        <div class="timestamp" title={absoluteTimestamp(op.timestamp)}>
          {formatTimestamp(op.timestamp)}
        </div>
      </div>
    </div>
  </div>
{:else if op.type === "assign"}
  <div class="timeline-item txt-body-m-regular">
    <div class="icon">
      <Icon name="avatar-incognito" />
    </div>
    <div class="wrapper">
      {#if !hideAuthor}<NodeId {...authorForNodeId(op.author)} />{/if}
      <div class="summary-line">
        {#if op.previous && op.previous.type === op.type}
          {@const changed = itemDiff<Author>(
            op.previous?.assignees ?? [],
            op.assignees,
          )}
          {#if changed.added.length}
            <span class="summary-secondary">assigned</span>
            {#each changed.added as assignee}
              <NodeId {...authorForNodeId(assignee)} />
            {/each}
          {/if}
          {#if changed.removed.length}
            <span class="summary-secondary">unassigned</span>
            {#each changed.removed as assignee}
              <NodeId {...authorForNodeId(assignee)} />
            {/each}
          {/if}
        {:else}
          <span class="txt-body-m-medium">assigned</span>
          {#each op.assignees as assignee}
            <NodeId {...authorForNodeId(assignee)} />
          {/each}
        {/if}
      </div>
      <div class="meta">
        <div class="timestamp" title={absoluteTimestamp(op.timestamp)}>
          {formatTimestamp(op.timestamp)}
        </div>
      </div>
    </div>
  </div>
{:else if op.type === "merge"}
  <div class="timeline-item txt-body-m-regular merge-badge">
    <div class="icon" style:color="var(--color-text-brand)">
      <Icon name="patch-merged" />
    </div>
    <div class="wrapper">
      {#if !hideAuthor}<NodeId {...authorForNodeId(op.author)} />{/if}
      <div class="summary-line">
        <span class="txt-body-m-medium">merged patch</span>
        {#if targetBranch}
          into <b>{targetBranch}</b>
        {/if}
      </div>
      <div class="meta">
        <Id id={op.revision} clipboard={op.revision} />
        <div class="timestamp" title={absoluteTimestamp(op.timestamp)}>
          {formatTimestamp(op.timestamp)}
        </div>
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
        {#if !hideAuthor}<NodeId {...authorForNodeId(op.author)} />{/if}
        <div class="summary-line">
          <span class="summary-secondary">changed title</span>
          <s class="summary-secondary">{op.previous.title}</s>
          <span class="summary-secondary">→</span>
          <span class="txt-body-m-medium summary-content">{op.title}</span>
        </div>
        <div class="meta">
          <div class="timestamp" title={absoluteTimestamp(op.timestamp)}>
            {formatTimestamp(op.timestamp)}
          </div>
        </div>
      </div>
    </div>
  {/if}
{:else if op.type === "review"}
  {@const hasSummary = !!op.summary && op.summary.trim() !== ""}
  {@const isToggleable = onToggle !== undefined}
  {#if op.verdict === "accept"}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <div
      class="timeline-item txt-body-m-regular verdict-accept"
      class:verdict-toggleable={isToggleable}
      role={isToggleable ? "button" : undefined}
      tabindex={isToggleable ? 0 : undefined}
      onclick={isToggleable ? onToggle : undefined}>
      <div class="icon" style:color="var(--color-feedback-success-text)">
        {#if isToggleable}
          <span class="icon-stack">
            <span class="icon-default"><Icon name="thumbs-up" /></span>
            <span class="icon-hover">
              <Icon name={expanded ? "collapse-vertical" : "expand-vertical"} />
            </span>
          </span>
        {:else}
          <Icon name="thumbs-up" />
        {/if}
      </div>
      <div class="wrapper">
        {#if !hideAuthor}<NodeId {...authorForNodeId(op.author)} />{/if}
        <div class="summary-line">
          <span class="summary-secondary">accepted revision</span>
        </div>
        <div class="meta">
          <Id id={op.revision} clipboard={op.revision} />
          <div class="timestamp" title={absoluteTimestamp(op.timestamp)}>
            {formatTimestamp(op.timestamp)}
          </div>
        </div>
        {#if hasSummary && (expanded || !isToggleable)}
          <div
            class="verdict-summary txt-body-m-medium"
            transition:slide={{ duration: 180 }}>
            {op.summary}
          </div>
        {/if}
      </div>
    </div>
  {:else if op.verdict === "reject"}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <div
      class="timeline-item txt-body-m-regular verdict-reject"
      class:verdict-toggleable={isToggleable}
      role={isToggleable ? "button" : undefined}
      tabindex={isToggleable ? 0 : undefined}
      onclick={isToggleable ? onToggle : undefined}>
      <div class="icon" style:color="var(--color-feedback-error-text)">
        {#if isToggleable}
          <span class="icon-stack">
            <span class="icon-default"><Icon name="stop" /></span>
            <span class="icon-hover">
              <Icon name={expanded ? "collapse-vertical" : "expand-vertical"} />
            </span>
          </span>
        {:else}
          <Icon name="stop" />
        {/if}
      </div>
      <div class="wrapper">
        {#if !hideAuthor}<NodeId {...authorForNodeId(op.author)} />{/if}
        <div class="summary-line">
          <span class="summary-secondary">rejected revision</span>
        </div>
        <div class="meta">
          <Id id={op.revision} clipboard={op.revision} />
          <div class="timestamp" title={absoluteTimestamp(op.timestamp)}>
            {formatTimestamp(op.timestamp)}
          </div>
        </div>
        {#if hasSummary && (expanded || !isToggleable)}
          <div
            class="verdict-summary txt-body-m-medium"
            transition:slide={{ duration: 180 }}>
            {op.summary}
          </div>
        {/if}
      </div>
    </div>
  {:else if op.verdict === undefined}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <div
      class="timeline-item txt-body-m-regular"
      class:toggleable={isToggleable}
      role={isToggleable ? "button" : undefined}
      tabindex={isToggleable ? 0 : undefined}
      onclick={isToggleable ? onToggle : undefined}>
      <div class="icon">
        {#if isToggleable}
          <span class="icon-stack">
            <span class="icon-default"><Icon name="comment" /></span>
            <span class="icon-hover">
              <Icon name={expanded ? "collapse-vertical" : "expand-vertical"} />
            </span>
          </span>
        {:else}
          <Icon name="comment" />
        {/if}
      </div>
      <div class="wrapper">
        {#if !hideAuthor}<NodeId {...authorForNodeId(op.author)} />{/if}
        <div class="summary-line">
          <span class="txt-body-m-medium">reviewed revision</span>
        </div>
        <div class="meta">
          <Id id={op.revision} clipboard={op.revision} />
          <div class="timestamp" title={absoluteTimestamp(op.timestamp)}>
            {formatTimestamp(op.timestamp)}
          </div>
        </div>
      </div>
    </div>
  {/if}
{/if}
