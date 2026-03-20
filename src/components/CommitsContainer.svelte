<script lang="ts">
  import type { Snippet } from "svelte";

  import Button from "@app/components/Button.svelte";
  import Icon from "@app/components/Icon.svelte";

  interface Props {
    leftHeader: Snippet;
    children: Snippet;
  }

  const { leftHeader, children }: Props = $props();

  let expanded = $state(true);
</script>

<style>
  .header {
    display: flex;
    align-items: center;
    height: 2.5rem;
    padding-left: 0.25rem;
    font: var(--txt-body-m-regular);
  }

  .left {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }
</style>

<div
  style:border="1px solid var(--color-border-subtle)"
  style:border-radius="var(--border-radius-md)"
  style:display="flex"
  style:align-items="flex-start"
  style:background-color="var(--color-surface-canvas)"
  style:flex-direction="column">
  <div class="header" class:collapsed={!expanded}>
    <div class="left">
      <Button
        variant="naked"
        onclick={async () => {
          expanded = !expanded;
        }}>
        <Icon name={expanded ? "chevron-down" : "chevron-right"} />
      </Button>
      {@render leftHeader()}
    </div>
  </div>

  {#if expanded}
    <div
      style:width="100%"
      style:border-top="1px solid var(--color-border-subtle)">
      {@render children()}
    </div>
  {/if}
</div>
