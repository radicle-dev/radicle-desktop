<script lang="ts">
  import Icon from "@app/components/Icon.svelte";

  interface Props {
    tooltip?: string;
  }

  const { tooltip = "Delegate" }: Props = $props();
</script>

<style>
  .delegate {
    display: inline-flex;
    align-items: center;
    flex-shrink: 0;
    color: var(--color-text-tertiary);
  }
  /* Tighten the gap when the badge follows another element, e.g. a NodeId,
     in a flex row: the icon's own whitespace makes a full gap look loose. */
  .delegate:not(:first-child) {
    margin-left: -0.25rem;
  }
</style>

<span class="delegate" title={tooltip}>
  <Icon name="badge" />
</span>
