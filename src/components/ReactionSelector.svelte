<script lang="ts">
  import type { Reaction } from "@bindings/cob/Reaction";

  import { createEventDispatcher } from "svelte";

  import Border from "./Border.svelte";
  import Icon from "./Icon.svelte";
  import Popover from "./Popover.svelte";

  export let reactions: Reaction[] | undefined = undefined;
  export let popoverPositionBottom: string | undefined = undefined;
  export let popoverPositionRight: string | undefined = undefined;
  export let popoverPositionLeft: string | undefined = undefined;

  const availableReactions = ["👍", "👎", "😄", "🎉", "🙁", "🚀", "👀"];

  const dispatch = createEventDispatcher<{
    select: Reaction;
  }>();
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
            onclick={() => {
              dispatch(
                "select",
                lookedUpReaction || { emoji: reaction, authors: [] },
              );
            }}>
            {reaction}
          </button>
        {/each}
      </div>
    </Border>
  {/snippet}
</Popover>
