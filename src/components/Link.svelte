<script lang="ts" strictEvents>
  import type { Route } from "@app/lib/router/definitions";

  import { createEventDispatcher } from "svelte";
  import { push, routeToPath } from "@app/lib/router";

  export let route: Route;
  export let disabled: boolean = false;

  const dispatch = createEventDispatcher<{
    afterNavigate: null;
  }>();

  function navigateToRoute(event: MouseEvent): void {
    event.preventDefault();
    if (disabled) {
      return;
    }

    void push(route);
    dispatch("afterNavigate");
  }
</script>

<style>
  a {
    color: var(--color-fill-secondary);
    text-decoration: none;
  }
  a:hover {
    text-decoration: underline;
    text-decoration-thickness: 1px;
    text-underline-offset: 2px;
  }
</style>

<a on:click={navigateToRoute} href={routeToPath(route)}>
  <slot />
</a>
