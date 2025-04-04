<script lang="ts">
  import type { Reaction } from "@bindings/cob/Reaction";

  import Border from "./Border.svelte";
  import Icon from "./Icon.svelte";
  import Popover from "./Popover.svelte";
  import { twemoji } from "@app/lib/utils";

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
    <Icon ariaLabel="toggle-reaction-selector" name="face" {onclick} />
  {/snippet}
  {#snippet popover()}
    <Border variant="ghost">
      <div class="selector">
        {#each availableReactions as reaction}
          {@const lookedUpReaction = reactions?.find(
            ({ emoji }) => emoji === reaction,
          )}
          <button
            aria-label={`reaction-selector-${reaction}`}
            use:twemoji={{ exclude: ["21a9"] }}
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
