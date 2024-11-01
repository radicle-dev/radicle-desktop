<script lang="ts">
  import type { Author } from "@bindings/cob/Author";
  import type { Reaction } from "@bindings/cob/Reaction";

  interface Props {
    reactions: Reaction[];
    handleReaction?: (authors: Author[], reaction: string) => Promise<void>;
  }

  const { reactions, handleReaction }: Props = $props();

  function authorsToTooltip(authors: Author[]) {
    return authors.map(a => a.alias ?? a.did).join("\n");
  }
</script>

<style>
  .reactions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  .reaction {
    display: inline-flex;
    flex-direction: row;
    gap: 0.5rem;
    cursor: pointer;
  }
</style>

<div class="reactions">
  {#each reactions as { emoji, authors }}
    <div title={authorsToTooltip(authors)}>
      {#if handleReaction}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <div
          role="button"
          tabindex="0"
          class="reaction txt-tiny"
          onclick={async () => {
            if (handleReaction) {
              await handleReaction(authors, emoji);
            }
          }}>
          <span>{emoji}</span>
          <span>{authors.length}</span>
        </div>
      {:else}
        <div class="reaction txt-tiny" style="padding: 2px 4px;">
          <span>{emoji}</span>
          <span>{authors.length}</span>
        </div>
      {/if}
    </div>
  {/each}
</div>
