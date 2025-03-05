<script lang="ts" module>
  export type Action = "done" | "running" | "error";
</script>

<script lang="ts">
  import type { ComponentProps, Snippet } from "svelte";
  import Icon from "./Icon.svelte";

  interface Props {
    status?: Action;
    position: number;
    children: Snippet;
  }

  function iconName(status: Action): ComponentProps<typeof Icon>["name"] {
    if (status === "done") {
      return "checkmark";
    } else if (status === "error") {
      return "warning";
    } else {
      return "ellipsis";
    }
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
    <Icon name={iconName(status)} />{@render children()}
  </div>
{/if}
