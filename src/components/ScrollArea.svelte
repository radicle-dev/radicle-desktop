<script lang="ts" module>
  import { getContext } from "svelte";

  const SCROLL_VIEWPORT_CONTEXT = Symbol("scroll-viewport");

  export function getScrollViewport(): () => HTMLElement | undefined {
    return (
      getContext<() => HTMLElement | undefined>(SCROLL_VIEWPORT_CONTEXT) ??
      (() => undefined)
    );
  }
</script>

<script lang="ts">
  import type { Snippet } from "svelte";

  import { OverlayScrollbarsComponent } from "overlayscrollbars-svelte";
  import { setContext } from "svelte";

  interface Props {
    children: Snippet;
    style?: string;
    onScrollHalf?: () => void;
  }

  const {
    children,
    style = "height: 100%;",
    onScrollHalf = undefined,
  }: Props = $props();

  let viewport: HTMLElement | undefined = $state(undefined);
  setContext(SCROLL_VIEWPORT_CONTEXT, () => viewport);

  function shouldLoadMore(instance: {
    elements(): { target: HTMLElement };
  }): boolean {
    const el = instance.elements().target;
    const threshold = 200;

    return el.scrollTop + el.clientHeight >= el.scrollHeight - threshold;
  }

  const scrollHalfHandler = onScrollHalf
    ? (instance: Parameters<typeof shouldLoadMore>[0]) => {
        if (shouldLoadMore(instance)) {
          onScrollHalf();
        }
      }
    : undefined;
</script>

<OverlayScrollbarsComponent
  element="div"
  {style}
  options={{
    scrollbars: { theme: "global-os-theme-radicle", autoHide: "scroll" },
  }}
  events={{
    initialized: instance => {
      viewport = instance.elements().viewport;
      scrollHalfHandler?.(instance);
    },
    scroll: scrollHalfHandler,
    updated: scrollHalfHandler,
  }}
  defer>
  {@render children()}
</OverlayScrollbarsComponent>
