<script lang="ts">
  import type { Route } from "@app/lib/router/definitions";

  import { push, routeToPath } from "@app/lib/router";

  export let route: Route;
  export let disabled: boolean = false;

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
