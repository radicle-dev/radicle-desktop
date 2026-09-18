<script lang="ts">
  import HoverPopover from "@app/components/HoverPopover.svelte";
  import Icon from "@app/components/Icon.svelte";

  interface Props {
    tooltip?: string;
  }

  const { tooltip = "Delegate" }: Props = $props();
</script>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    flex-shrink: 0;
  }
  /* HoverPopover wraps its trigger in an inline-block container and a block
     button, whose line boxes push the icon off the centre line of whatever it
     sits beside. Neither needs a box of its own here. */
  .badge :global(.container),
  .badge :global([role="button"]) {
    display: inline-flex;
    align-items: center;
    line-height: 0;
  }
  /* Tighten the gap when the badge follows another element, e.g. a NodeId,
     in a flex row: the icon's own whitespace makes a full gap look loose. */
  .badge:not(:first-child) {
    margin-left: -0.25rem;
  }
  .delegate {
    display: inline-flex;
    align-items: center;
    color: var(--color-text-tertiary);
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
      <span class="delegate" aria-label={tooltip}>
        <Icon name="badge" />
      </span>
    {/snippet}
    {#snippet popover()}
      <span class="tooltip">{tooltip}</span>
    {/snippet}
  </HoverPopover>
</span>
