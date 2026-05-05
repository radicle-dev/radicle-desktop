<script lang="ts">
  import type { Commit } from "@bindings/repo/Commit";

  import CommitActivityItem from "@app/components/CommitActivityItem.svelte";
  import Icon from "@app/components/Icon.svelte";

  interface Props {
    commits: Commit[];
    rid: string;
    expandBatch?: number;
  }

  const { commits, rid, expandBatch = 10 }: Props = $props();

  let expanded = $state(0);

  const visible = $derived(commits.slice(0, expanded));
  const remaining = $derived(commits.length - expanded);
  const batchSize = $derived(Math.min(expandBatch, remaining));

  function showMore() {
    expanded = Math.min(expanded + expandBatch, commits.length);
  }
</script>

<style>
  .connector {
    width: 1px;
    height: 1rem;
    margin-left: 1.25rem;
    background-color: var(--color-border-subtle);
  }
  .collapsed {
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
  }
  .icon {
    padding-top: 0.1875rem;
    color: var(--color-text-secondary);
  }
  .wrapper {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.5rem;
  }
  .label {
    color: var(--color-text-tertiary);
  }
  .button {
    background: var(--color-surface-subtle);
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    padding: 0.125rem 0.5rem;
    cursor: pointer;
    color: var(--color-text-secondary);
  }
  .button:hover {
    color: var(--color-text-primary);
  }
</style>

{#each visible as commit, idx (commit.id)}
  <CommitActivityItem {commit} {rid} />
  {#if idx < visible.length - 1 || remaining > 0}
    <div class="connector"></div>
  {/if}
{/each}

{#if remaining > 0}
  <div class="collapsed txt-body-m-regular">
    <div class="icon">
      <Icon name="commit" />
    </div>
    <div class="wrapper">
      <span class="label">{remaining} commits</span>
      <button class="button txt-body-s-regular" onclick={showMore}>
        Show {batchSize} more
      </button>
    </div>
  </div>
{/if}
