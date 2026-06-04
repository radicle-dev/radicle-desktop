<script lang="ts">
  import type { Commit } from "@bindings/repo/Commit";

  import fuzzysort from "fuzzysort";

  interface Props {
    commits: Commit[];
    value: string;
    disabled?: boolean;
    onSelect: (oid: string) => void;
  }

  const { commits, value, disabled = false, onSelect }: Props = $props();

  let query = $state(value);
  let open = $state(false);

  // Keep the typed value in sync when the parent overwrites it (e.g. tag
  // selection auto-fills the OID). Without this the picker would keep
  // showing the user's stale input.
  $effect(() => {
    query = value;
  });

  const results = $derived(
    fuzzysort.go(query, commits, {
      keys: ["id", "summary"],
      threshold: 0.5,
      all: true,
      limit: 20,
    }),
  );

  function pick(oid: string) {
    query = oid;
    open = false;
    onSelect(oid);
  }
</script>

<style>
  .wrapper {
    position: relative;
  }
  input {
    width: 100%;
    padding: 0.4rem 0.5rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    background-color: var(--color-surface-canvas);
    font: var(--txt-body-m-regular);
  }
  .menu {
    position: absolute;
    z-index: 10;
    top: calc(100% + 0.25rem);
    left: 0;
    right: 0;
    max-height: 18rem;
    overflow-y: auto;
    background-color: var(--color-surface-canvas);
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    box-shadow: var(--elevation-low);
  }
  .item {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
    padding: 0.5rem 0.75rem;
    cursor: pointer;
    text-align: left;
    background: none;
    border: none;
    width: 100%;
    border-bottom: 1px solid var(--color-border-subtle);
  }
  .item:last-child {
    border-bottom: none;
  }
  .item:hover {
    background-color: var(--color-surface-subtle);
  }
  .summary {
    font: var(--txt-body-m-regular);
    color: var(--color-text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .oid {
    font: var(--txt-body-s-mono);
    color: var(--color-text-secondary);
  }
  .empty {
    padding: 0.75rem;
    color: var(--color-text-secondary);
    font: var(--txt-body-s-regular);
  }
</style>

<div class="wrapper">
  <input
    type="text"
    placeholder="Search commits or paste an OID"
    bind:value={query}
    oninput={() => onSelect(query)}
    onfocus={() => (open = true)}
    onblur={() => setTimeout(() => (open = false), 150)}
    {disabled} />
  {#if open && !disabled}
    <div class="menu">
      {#each results as result}
        <button
          type="button"
          class="item"
          onmousedown={e => {
            e.preventDefault();
            pick(result.obj.id);
          }}>
          <span class="summary">{result.obj.summary}</span>
          <span class="oid">{result.obj.id.slice(0, 12)}…</span>
        </button>
      {:else}
        <div class="empty">No matching commits</div>
      {/each}
    </div>
  {/if}
</div>
