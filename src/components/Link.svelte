<script lang="ts">
  import type { Snippet } from "svelte";
  import type { Route } from "@app/lib/router/definitions";

  import { push, routeToPath } from "@app/lib/router";

  export let children: Snippet;
  export let route: Route;
  export let disabled: boolean = false;
  export let variant: "active" | "regular" | "tab" = "regular";

  function navigateToRoute(event: MouseEvent): void {
    event.preventDefault();
    if (disabled) {
      return;
    }

    void push(route);
  }
</script>

<style>
  a {
    color: var(--color-foreground-contrast);
    text-decoration: none;
  }
  .regular:hover {
    text-decoration: underline;
    text-decoration-thickness: 1px;
    text-underline-offset: 2px;
  }

  .tab {
    display: flex;
    width: 100%;
    justify-content: space-between;
    align-items: center;
    padding: 4px 4px 4px 10px;
    clip-path: var(--2px-corner-fill);
  }

  .tab:hover {
    background-color: var(--color-fill-ghost);
  }

  .active {
    background-color: var(--color-fill-ghost);
    display: flex;
    width: 100%;
    justify-content: space-between;
    align-items: center;
    padding: 4px 4px 4px 10px;
    clip-path: var(--2px-corner-fill);
  }
</style>

<a
  onclick={navigateToRoute}
  href={routeToPath(route)}
  class:regular={variant === "regular"}
  class:active={variant === "active"}
  class:tab={variant === "tab"}>
  {@render children()}
</a>
