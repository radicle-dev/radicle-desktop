<script lang="ts">
  import type { Snippet } from "svelte";

  import { tick } from "svelte";

  import Button from "@app/components/Button.svelte";
  import Icon from "@app/components/Icon.svelte";

  interface Props {
    children: Snippet;
    expandable?: boolean;
    expanded?: boolean;
    leftHeader?: Snippet;
    rightHeader?: Snippet;
    sticky?: boolean;
    border?: boolean;
  }

  /* eslint-disable prefer-const */
  let {
    children,
    expanded = true,
    leftHeader,
    rightHeader,
    sticky = true,
    expandable = true,
    border = true,
  }: Props = $props();
  /* eslint-enable prefer-const */

  let header: HTMLElement | undefined = $state();
</script>

<style>
  .header {
    display: flex;
    align-items: center;
    height: 2.5rem;
    padding-left: 0.25rem;
    z-index: 2;
    font: var(--txt-body-m-regular);
    position: relative;
    background-color: var(--color-surface-canvas);
    border-top-left-radius: var(--border-radius-md);
    border-top-right-radius: var(--border-radius-md);
  }

  .sticky {
    position: sticky;
    top: 0;
  }

  .left {
    display: flex;
    gap: 0.5rem;
    margin-right: 1rem;
    align-items: center;
  }

  .container {
    position: relative;
    overflow-x: auto;
    z-index: 1;
    border-top: none;
    border-bottom-left-radius: var(--border-radius-md);
    border-bottom-right-radius: var(--border-radius-md);
  }
</style>

<div
  class="header"
  class:sticky
  class:collapsed={!expanded}
  bind:this={header}
  style:border={border ? "1px solid var(--color-border-subtle)" : undefined}
  style:border-bottom={border
    ? "undefined"
    : "1px solid var(--color-border-subtle)"}>
  <div class="left">
    {#if expandable}
      <Button
        variant="naked"
        onclick={async () => {
          expanded = !expanded;
          if (!expanded && header) {
            await tick();
            header.scrollIntoView({ behavior: "smooth", block: "nearest" });
          }
        }}>
        <Icon name={expanded ? "chevron-down" : "chevron-right"} />
      </Button>
    {/if}
    {@render leftHeader?.()}
  </div>
  {#if rightHeader}
    <div
      class="global-flex"
      style:gap="1rem"
      style:margin-left="auto"
      style:margin-right="1rem">
      {@render rightHeader()}
    </div>
  {/if}
</div>

{#if expanded}
  <div
    class="container"
    style:border={border ? "1px solid var(--color-border-subtle)" : "undefined"}
    style:border-top="none">
    {@render children()}
  </div>
{/if}
