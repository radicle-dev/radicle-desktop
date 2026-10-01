<script lang="ts">
  import HoverPopover from "@app/components/HoverPopover.svelte";

  interface Props {
    tooltip?: string;
  }

  const { tooltip = "You are the author" }: Props = $props();
</script>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    flex-shrink: 0;
  }
  /* Same reset as DelegateBadge: keep HoverPopover's wrappers from pushing
     the chip off the centre line of the row it sits in. */
  .badge :global(.container),
  .badge :global([role="button"]) {
    display: inline-flex;
    align-items: center;
    line-height: 0;
  }
  .you {
    font: var(--txt-body-s-regular);
    color: var(--color-text-tertiary);
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    padding: 0 0.375rem;
    white-space: nowrap;
  }
  .tooltip {
    font: var(--txt-body-s-regular);
    color: var(--color-text-primary);
    white-space: nowrap;
  }
</style>

<span class="badge">
  <HoverPopover stylePadding="0.25rem 0.5rem">
    {#snippet toggle()}
      <span class="you" aria-label={tooltip}>You</span>
    {/snippet}
    {#snippet popover()}
      <span class="tooltip">{tooltip}</span>
    {/snippet}
  </HoverPopover>
</span>
