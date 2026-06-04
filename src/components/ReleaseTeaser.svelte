<script lang="ts">
  import type { ReleaseFilter } from "@app/views/repo/router";
  import type { Release } from "@bindings/cob/release/Release";

  import { push } from "@app/lib/router";
  import { authorForNodeId, formatTimestamp } from "@app/lib/utils";

  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import NodeId from "@app/components/NodeId.svelte";

  interface Props {
    release: Release;
    rid: string;
    filter: ReleaseFilter;
  }

  const { release, rid, filter }: Props = $props();
</script>

<style>
  .teaser {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 0.5rem;
    min-height: 5rem;
    background-color: var(--color-surface-canvas);
    padding: 1rem 1.25rem;
    cursor: pointer;
    font: var(--txt-body-l-regular);
    word-break: break-word;
    width: 100%;
  }
  .teaser:hover {
    background-color: var(--color-surface-subtle);
  }
  .teaser:first-of-type {
    border-radius: var(--border-radius-sm) var(--border-radius-sm) 0 0;
  }
  .teaser:last-of-type {
    border-radius: 0 0 var(--border-radius-sm) var(--border-radius-sm);
  }
  .teaser:only-of-type {
    border-radius: var(--border-radius-sm);
  }
  .left {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
  }
  .body {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    align-items: flex-start;
  }
  .title {
    font: var(--txt-body-l-semibold);
    color: var(--color-text-primary);
  }
  .meta {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.375rem;
    color: var(--color-text-secondary);
    font: var(--txt-body-m-regular);
  }
  .tag-pill {
    font: var(--txt-body-s-mono);
    color: var(--color-text-secondary);
    background-color: var(--color-surface-subtle);
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    padding: 0.0625rem 0.375rem;
  }
  .right {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
    white-space: nowrap;
  }
  .count-chip {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    height: 1.5rem;
    padding: 0 0.5rem;
    color: var(--color-text-tertiary);
  }
</style>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  tabindex="0"
  role="button"
  class="teaser"
  onclick={() => {
    void push({
      resource: "repo.release",
      rid,
      release: release.id,
      filter,
    });
  }}>
  <div class="left">
    <div class="global-chip" style:padding="0">
      <Icon name="commit" />
    </div>
    <div class="body">
      <span class="title">
        {release.tagName ??
          release.commitSummary ??
          `Release ${release.oid.slice(0, 7)}`}
      </span>
      <div class="meta">
        <NodeId {...authorForNodeId(release.creator)} />
        released
        <Id id={release.id} clipboard={release.id} ariaLabel="Release ID" />
        from commit
        <Id id={release.oid} clipboard={release.oid} ariaLabel="Commit OID" />
        {#if release.tagName}
          <span class="tag-pill">{release.tagName}</span>
        {/if}
        {formatTimestamp(release.timestamp * 1000)}
      </div>
    </div>
  </div>

  <div class="right">
    <span class="count-chip">
      <Icon name="archive" />
      {release.artifacts.length}
    </span>
  </div>
</div>
