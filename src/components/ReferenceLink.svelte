<script lang="ts">
  import type { Config } from "@bindings/config/Config";

  import { cachedConfig, cachedRepoById } from "@app/lib/invoke";
  import { describeLink, referenceRoute } from "@app/lib/mentions";
  import { formatReference, parseReference } from "@app/lib/radUri";
  import { push, routeToPath } from "@app/lib/router";
  import { explorerLink } from "@app/lib/utils";

  import ExplorerMenu, {
    keepSelection,
    type MenuPosition,
    menuPosition,
  } from "@app/components/ExplorerMenu.svelte";
  import Icon from "@app/components/Icon.svelte";

  interface Props {
    href: string;
    label: string;
  }

  const { href, label }: Props = $props();

  const reference = $derived(parseReference(href));
  const icon = $derived(reference && describeLink(reference, "")?.icon);
  let config: Config | undefined = $state(undefined);
  const url = $derived(
    reference && config ? explorerLink(reference, config) : undefined,
  );

  $effect(() => {
    if (!reference || config) return;
    let cancelled = false;
    void cachedConfig()
      .then(result => {
        if (!cancelled) config = result;
      })
      .catch(console.error);
    return () => {
      cancelled = true;
    };
  });

  const route = $derived(reference && referenceRoute(reference));
  let local = $state(false);
  $effect(() => {
    if (!route) return;
    let cancelled = false;
    void cachedRepoById(route.rid)
      .then(repo => {
        if (!cancelled) local = repo !== null;
      })
      .catch(() => {
        if (!cancelled) local = false;
      });
    return () => {
      cancelled = true;
    };
  });
  const inApp = $derived(local && route !== undefined);
  const linkHref = $derived(inApp && route ? routeToPath(route) : url);

  function handleClick(event: MouseEvent) {
    if (!inApp || !route) return;
    event.preventDefault();
    void push(route);
  }

  let menu: MenuPosition | undefined = $state(undefined);

  function openMenu(event: MouseEvent) {
    if (url) menu = menuPosition(event);
  }
</script>

<style>
  .icon {
    display: inline-flex;
    vertical-align: middle;
    margin-right: 0.25rem;
    color: var(--color-text-tertiary);
  }
</style>

<a
  href={linkHref}
  target={!inApp && url ? "_blank" : undefined}
  rel={!inApp && url ? "noopener noreferrer" : undefined}
  title={href}
  onclick={handleClick}
  onmousedown={keepSelection}
  oncontextmenu={openMenu}>
  {#if icon}
    <span class="icon"><Icon name={icon} /></span>
  {/if}{label}
</a>

{#if menu && url && config}
  <ExplorerMenu
    x={menu.x}
    y={menu.y}
    target={menu.target}
    {url}
    uri={reference ? formatReference(reference) : href}
    {config}
    onclose={() => (menu = undefined)} />
{/if}
