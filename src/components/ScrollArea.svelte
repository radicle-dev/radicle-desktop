<script lang="ts">
  import type { Snippet } from "svelte";

  import { OverlayScrollbarsComponent } from "overlayscrollbars-svelte";

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
</script>

<OverlayScrollbarsComponent
  element="div"
  {style}
  options={{
    scrollbars: { theme: "global-os-theme-radicle", autoHide: "scroll" },
  }}
  events={onScrollHalf
    ? {
        scroll: instance => {
          const el = instance.elements().target;
          if (el.scrollTop + el.clientHeight >= el.scrollHeight / 2) {
            onScrollHalf();
          }
        },
      }
    : undefined}
  defer>
  {@render children()}
</OverlayScrollbarsComponent>
