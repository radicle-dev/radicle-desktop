<script lang="ts">
  import type { Reaction } from "@bindings/cob/Reaction";

  import Border from "./Border.svelte";
  import Icon from "./Icon.svelte";
  import Popover from "./Popover.svelte";

  interface Props {
    reactions?: Reaction[];
    popoverPositionBottom?: string;
    popoverPositionRight?: string;
    popoverPositionLeft?: string;
    select: (reaction: Reaction) => Promise<void>;
  }

  const {
    reactions,
    popoverPositionBottom,
    popoverPositionRight,
    popoverPositionLeft,
    select,
  }: Props = $props();

  const availableReactions = ["👍", "👎", "😄", "🎉", "🙁", "🚀", "👀"];
</script>

<style>
  .selector {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  button {
    cursor: pointer;
    border: 0;
    background: none;
    height: 24px;
    clip-path: var(--1px-corner-fill);
    margin: 0;
  }

  button:hover,
  button.active {
    background-color: var(--color-fill-ghost);
  }
</style>

<Popover
  {popoverPositionBottom}
  {popoverPositionRight}
  {popoverPositionLeft}
  popoverPadding="0">
  {#snippet toggle(onclick)}
    <Icon name="face" {onclick} styleCursor="pointer" />
  {/snippet}
  {#snippet popover()}
    <Border variant="ghost">
      <div class="selector">
        {#each availableReactions as reaction}
          {@const lookedUpReaction = reactions?.find(
            ({ emoji }) => emoji === reaction,
          )}
          <button
            class:active={Boolean(lookedUpReaction)}
            onclick={() =>
              select(lookedUpReaction || { emoji: reaction, authors: [] })}>
            {reaction}
          </button>
        {/each}
      </div>
    </Border>
  {/snippet}
</Popover>
