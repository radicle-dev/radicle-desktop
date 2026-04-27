<script lang="ts">
  import { getScrollViewport } from "@app/components/ScrollArea.svelte";

  interface Props {
    onIntersect: () => void | Promise<void>;
    disabled?: boolean;
    rootMargin?: string;
  }

  const {
    onIntersect,
    disabled = false,
    rootMargin = "500%",
  }: Props = $props();

  const getViewport = getScrollViewport();
  let sentinel: HTMLElement | undefined = $state();
  let firing = false;

  $effect(() => {
    if (disabled) return;
    const viewport = getViewport();
    if (!viewport || !sentinel) return;
    if (!("IntersectionObserver" in window)) return;

    const observer = new IntersectionObserver(
      entries => {
        for (const entry of entries) {
          if (!entry.isIntersecting || firing) continue;
          firing = true;
          void Promise.resolve(onIntersect()).finally(() => {
            firing = false;
          });
        }
      },
      { root: viewport, rootMargin },
    );

    observer.observe(sentinel);
    return () => observer.disconnect();
  });
</script>

<div bind:this={sentinel} aria-hidden="true" style="height: 1px;"></div>
