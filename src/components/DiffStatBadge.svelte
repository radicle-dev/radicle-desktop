<script lang="ts">
  import type { Stats } from "@bindings/diff/Stats";

  interface Props {
    stats: Stats;
  }

  const { stats }: Props = $props();
</script>

<style>
  .diff-stat-badge {
    display: flex;
    clip-path: var(--1px-corner-fill);
    height: 1.5rem;
  }

  .insertions {
    display: flex;
    padding: 0 0.5rem;
    align-items: center;
    color: var(--color-foreground-success);
    background-color: var(--color-fill-diff-green-light);
  }
  .deletions {
    display: flex;
    padding: 0 0.5rem;
    align-items: center;
    color: var(--color-foreground-red);
    background-color: var(--color-fill-diff-red-light);
  }
</style>

<div class="diff-stat-badge txt-mono txt-semibold txt-small">
  <div class="insertions">+{stats.insertions}</div>
  <div class="deletions">-{stats.deletions}</div>
</div>
