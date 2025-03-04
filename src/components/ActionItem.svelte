<script lang="ts">
  import type { Snippet } from "svelte";
  import Icon from "./Icon.svelte";

  type Actions = "done" | "running";

  interface Props {
    status?: Actions;
    position: number;
    children: Snippet;
  }

  const { status, position, children }: Props = $props();
</script>

{#if status === undefined}
  <div class="global-flex">
    <span class="txt-monospace">{position}.</span>
    {@render children()}
  </div>
{:else}
  <div class="global-flex">
    <Icon
      name={status === "done" ? "checkmark" : "ellipsis"} />{@render children()}
  </div>
{/if}
