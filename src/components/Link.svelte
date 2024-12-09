<script lang="ts">
  import type { Snippet } from "svelte";
  import type { Route } from "@app/lib/router/definitions";

  import { push, routeToPath } from "@app/lib/router";

  interface Props {
    children: Snippet;
    route: Route;
    disabled?: boolean;
    underline?: boolean;
    styleWidth?: string;
    styleColor?: string;
  }

  const {
    children,
    route,
    disabled = false,
    underline = true,
    styleWidth,
    styleColor,
  }: Props = $props();

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
  .underline:hover {
    text-decoration: underline;
    text-decoration-thickness: 1px;
    text-underline-offset: 2px;
  }
</style>

<a
  onclick={navigateToRoute}
  href={routeToPath(route)}
  class:underline
  style:color={styleColor}
  style:width={styleWidth}>
  {@render children()}
</a>
