<script lang="ts">
  import type { Snippet } from "svelte";

  import { autoUpdate, computePosition, flip, shift } from "@floating-ui/dom";
  import { onMount } from "svelte";

  import { portal } from "@app/lib/portal";

  import { getScrollViewport } from "@app/components/ScrollArea.svelte";

  interface Props {
    x: number;
    y: number;
    target?: HTMLElement;
    onclose: () => void;
    children: Snippet;
  }

  const { x, y, target, onclose, children }: Props = $props();

  const getViewport = getScrollViewport();

  function findScrollParent(el: HTMLElement | undefined): HTMLElement {
    let node: HTMLElement | null = el ?? null;
    while (node && node !== document.body) {
      const style = getComputedStyle(node);
      const overflowY = style.overflowY;
      const overflowX = style.overflowX;
      const scrolls =
        (overflowY === "auto" ||
          overflowY === "scroll" ||
          overflowX === "auto" ||
          overflowX === "scroll") &&
        (node.scrollHeight > node.clientHeight ||
          node.scrollWidth > node.clientWidth);
      if (scrolls) return node;
      node = node.parentElement;
    }
    return document.scrollingElement as HTMLElement;
  }

  function menuItems(): HTMLElement[] {
    if (!menuEl) return [];
    return Array.from(
      menuEl.querySelectorAll<HTMLElement>('[role="menuitem"]'),
    );
  }

  function focusItem(index: number) {
    const items = menuItems();
    if (items.length === 0) return;
    const wrapped = ((index % items.length) + items.length) % items.length;
    items[wrapped].focus();
  }

  function onMenuKeydown(ev: KeyboardEvent) {
    const items = menuItems();
    if (items.length === 0) return;
    const current = items.findIndex(el => el === document.activeElement);
    switch (ev.key) {
      case "ArrowDown":
        ev.preventDefault();
        focusItem(current + 1);
        break;
      case "ArrowUp":
        ev.preventDefault();
        focusItem(current - 1);
        break;
      case "Home":
        ev.preventDefault();
        focusItem(0);
        break;
      case "End":
        ev.preventDefault();
        focusItem(items.length - 1);
        break;
    }
  }

  function onMenuClick(ev: MouseEvent) {
    const el = ev.target as HTMLElement | null;
    if (el?.closest('[role="menuitem"]')) onclose();
  }

  let menuEl: HTMLDivElement | undefined = $state();

  const virtualEl = {
    getBoundingClientRect: () => ({
      x,
      y,
      top: y,
      left: x,
      right: x,
      bottom: y,
      width: 0,
      height: 0,
      toJSON: () => undefined,
    }),
  };

  function onWindowClick(ev: MouseEvent | TouchEvent) {
    if (!menuEl) return;
    if (!ev.composedPath().includes(menuEl)) onclose();
  }

  function onWindowKeydown(ev: KeyboardEvent) {
    if (ev.key === "Escape") onclose();
  }

  onMount(() => {
    const ac = new AbortController();
    const { signal } = ac;

    requestAnimationFrame(() => {
      if (signal.aborted) return;
      window.addEventListener("click", onWindowClick, { signal });
      window.addEventListener("contextmenu", onWindowClick, { signal });
      window.addEventListener("mousedown", onWindowClick, { signal });
      window.addEventListener("keydown", onWindowKeydown, { signal });
      menuEl?.focus({ preventScroll: true });
    });

    const scrollParent = getViewport() ?? findScrollParent(target);
    const preventScroll = (ev: Event) => ev.preventDefault();
    scrollParent.addEventListener("wheel", preventScroll, {
      passive: false,
      signal,
    });
    scrollParent.addEventListener("touchmove", preventScroll, {
      passive: false,
      signal,
    });

    return () => ac.abort();
  });

  $effect(() => {
    if (!menuEl) return;
    return autoUpdate(virtualEl, menuEl, () => {
      void computePosition(virtualEl, menuEl!, {
        strategy: "fixed",
        placement: "bottom-start",
        middleware: [flip(), shift({ padding: 8 })],
      }).then(({ x: px, y: py }) => {
        if (menuEl) {
          menuEl.style.left = `${px}px`;
          menuEl.style.top = `${py}px`;
          menuEl.style.visibility = "visible";
        }
      });
    });
  });
</script>

<style>
  .menu {
    position: fixed;
    top: 0;
    left: 0;
    visibility: hidden;
    z-index: 100;
    min-width: 12rem;
    padding: 0.25rem;
    background-color: var(--color-surface-canvas);
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-md);
    box-shadow: var(--elevation-low);
  }
  .menu:focus {
    outline: none;
  }
</style>

<div
  use:portal
  bind:this={menuEl}
  class="menu"
  role="menu"
  tabindex="-1"
  onclick={onMenuClick}
  onkeydown={onMenuKeydown}
  oncontextmenu={e => e.preventDefault()}>
  {@render children()}
</div>
