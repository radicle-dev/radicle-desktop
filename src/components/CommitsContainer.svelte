<script lang="ts">
  import type { Snippet } from "svelte";

  import Border from "./Border.svelte";
  import Icon from "./Icon.svelte";
  import NakedButton from "./NakedButton.svelte";

  interface Props {
    leftHeader: Snippet;
    children: Snippet;
    expanded: boolean;
  }

  /* eslint-disable prefer-const */
  let { leftHeader, children, expanded }: Props = $props();
  /* eslint-enable prefer-const */
</script>

<style>
  .header {
    display: flex;
    align-items: center;
    height: 2rem;
    padding-left: 0.25rem;
    font-size: var(--font-size-small);
    background-color: var(--color-background-default);
  }

  .left {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }
  .divider {
    width: calc(100% + 4px);
    position: relative;
    top: -6px;
    left: -2px;
    z-index: 1;
    height: 2px;
    background-color: var(--color-fill-ghost);
  }
</style>

<Border
  variant="ghost"
  styleFlexDirection="column"
  styleAlignItems="flex-start">
  <div class="header" class:collapsed={!expanded}>
    <div class="left">
      <NakedButton
        stylePadding="0 4px"
        variant="ghost"
        onclick={async () => {
          expanded = !expanded;
        }}>
        <Icon name={expanded ? "chevron-down" : "chevron-right"} />
      </NakedButton>
      {@render leftHeader()}
    </div>
  </div>

  {#if expanded}
    <div class="divider"></div>
    <div style:width="100%">
      {@render children()}
    </div>
  {/if}
</Border>
