<script lang="ts">
  import type { Comment } from "@bindings/Comment";

  export let reactions: Comment["reactions"];
  export let handleReaction:
    | ((
        authors: Comment["reactions"][0]["authors"],
        reaction: string,
      ) => Promise<void>)
    | undefined;

  function authorsToTooltip(authors: Comment["reactions"][0]["authors"]) {
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
