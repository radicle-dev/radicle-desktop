<script lang="ts">
  import type { Config } from "@bindings/config/Config";

  import { cachedConfig, cachedRepoById } from "@app/lib/invoke";
  import { referenceRoute } from "@app/lib/mentions";
  import { filePath, formatReference, parseReference } from "@app/lib/radUri";
  import { push, routeToPath } from "@app/lib/router";
  import { explorerLink } from "@app/lib/utils";

  import ExplorerMenu from "@app/components/ExplorerMenu.svelte";
  import Icon from "@app/components/Icon.svelte";

  interface Props {
    /** A `rad:` or `did:` href that has no chip, such as a file. */
    href: string;
    label: string;
  }

  const { href, label }: Props = $props();

  const reference = $derived(parseReference(href));
  const file = $derived(reference && filePath(reference));
  let config: Config | undefined = $state(undefined);
  // Neither scheme has a handler outside the app, so this links to the
  // explorer when there is a page for it, and is inert otherwise.
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

  // Opened in-app when the repo is here, like a chip, and on the explorer
  // otherwise.
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

  let menu: { x: number; y: number; target: HTMLElement } | undefined =
    $state(undefined);

  // WebKit selects the word under a right click, which would highlight the
  // label behind the menu.
  function keepSelection(event: MouseEvent) {
    if (event.button === 2) event.preventDefault();
  }

  function openMenu(event: MouseEvent) {
    if (!url) return;
    event.preventDefault();
    event.stopPropagation();
    window.getSelection()?.removeAllRanges();
    menu = {
      x: event.clientX,
      y: event.clientY,
      target: event.currentTarget as HTMLElement,
    };
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
  {#if file}
    <span class="icon">
      <Icon name={file.path ? "document" : "folder"} />
    </span>
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
