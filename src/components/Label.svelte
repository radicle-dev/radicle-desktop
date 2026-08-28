<script lang="ts">
  import type { ComponentProps } from "svelte";

  import Icon from "@app/components/Icon.svelte";

  interface Props {
    label: string;
    icon?: ComponentProps<typeof Icon>["name"];
  }

  const { label, icon = undefined }: Props = $props();
</script>

<style>
  .label {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.25rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    height: 2rem;
    padding: 0 0.5rem;
    color: var(--color-text-tertiary);
    max-width: 10rem;
  }
</style>

<div class="label txt-body-m-regular">
  {#if icon}
    <Icon name={icon} />
  {/if}
  <div class="txt-overflow" title={label}>{label}</div>
</div>
