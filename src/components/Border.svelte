<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    children: Snippet;
    variant: "primary" | "secondary" | "ghost" | "float" | "danger" | "success";
    hoverable?: boolean;
    onclick?: (e: MouseEvent) => void;
    stylePosition?: string;
    stylePadding?: string;
    styleHeight?: string;
    styleMinHeight?: string;
    styleMinWidth?: string;
    styleWidth?: string;
    styleDisplay?: string;
    styleCursor?: "default" | "pointer" | "text";
    styleGap?: string;
    styleOverflow?: string;
    flatTop?: boolean;
    flatBottom?: boolean;
    styleBackgroundColor?: string;
    styleFlexDirection?: string;
    styleAlignItems?: string;
    styleJustifyContent?: string;
  }

  const {
    children,
    variant,
    hoverable = false,
    onclick,
    stylePadding,
    styleHeight,
    styleMinHeight,
    stylePosition,
    styleWidth,
    styleDisplay = "flex",
    styleCursor = "default",
    styleGap = "0.5rem",
    styleMinWidth,
    styleOverflow,
    flatTop = false,
    flatBottom = false,
    styleBackgroundColor = "var(--color-background-default)",
    styleFlexDirection = "row",
    styleAlignItems = "center",
    styleJustifyContent,
  }: Props = $props();

  const style = $derived(
    `--local-background-color: ${styleBackgroundColor};` +
      `--local-button-color-1: var(--color-fill-${variant});` +
      `--local-hover-background-color: ${hoverable ? "var(--color-background-float)" : styleBackgroundColor}`,
  );
</script>

<style>
  .container {
    -webkit-touch-callout: none;
    -webkit-user-select: none;
    user-select: none;

    flex: 1;
    column-gap: 0;
    row-gap: 0;
    display: grid;
    grid-template-columns: 2px 2px auto 2px 2px;
    grid-template-rows: 2px 2px auto 2px 2px;
    grid-template-areas:
      "p1-1 p1-2 p1-3 p1-4 p1-5"
      "p2-1 p2-2 p2-3 p2-4 p2-5"
      "p3-1 p3-2 p3-3 p3-4 p3-5"
      "p4-1 p4-2 p4-3 p4-4 p4-5"
      "p5-1 p5-2 p5-3 p5-4 p5-5";
  }

  .container:hover > .p2-3,
  .container:hover > .p3-2,
  .container:hover > .p3-3,
  .container:hover > .p3-4,
  .container:hover > .p4-3 {
    background-color: var(--local-hover-background-color);
  }

  .p1-1 {
    grid-area: p1-1;
    background-color: transparent;
  }
  .p1-2 {
    grid-area: p1-2;
    background-color: transparent;
  }
  .p1-3 {
    grid-area: p1-3;
    background-color: var(--local-button-color-1);
  }
  .p1-4 {
    grid-area: p1-4;
    background-color: transparent;
  }
  .p1-5 {
    grid-area: p1-5;
    background-color: transparent;
  }

  .p2-1 {
    grid-area: p2-1;
    background-color: transparent;
  }
  .p2-2 {
    grid-area: p2-2;
    background-color: var(--local-button-color-1);
  }
  .p2-3 {
    grid-area: p2-3;
    background-color: var(--local-background-color);
  }
  .p2-4 {
    grid-area: p2-4;
    background-color: var(--local-button-color-1);
  }
  .p2-5 {
    grid-area: p2-5;
    background-color: transparent;
  }

  .p3-1 {
    grid-area: p3-1;
    background-color: var(--local-button-color-1);
  }
  .p3-2 {
    grid-area: p3-2;
    background-color: var(--local-background-color);
  }
  .p3-3 {
    grid-area: p3-3;
    background-color: var(--local-background-color);
  }
  .p3-4 {
    grid-area: p3-4;
    background-color: var(--local-background-color);
  }
  .p3-5 {
    grid-area: p3-5;
    background-color: var(--local-button-color-1);
  }

  .p4-1 {
    grid-area: p4-1;
    background-color: transparent;
  }
  .p4-2 {
    grid-area: p4-2;
    background-color: var(--local-button-color-1);
  }
  .p4-3 {
    grid-area: p4-3;
    background-color: var(--local-background-color);
  }
  .p4-4 {
    grid-area: p4-4;
    background-color: var(--local-button-color-1);
  }
  .p4-5 {
    grid-area: p4-5;
    background-color: transparent;
  }

  .p5-1 {
    grid-area: p5-1;
    background-color: transparent;
  }
  .p5-2 {
    grid-area: p5-2;
    background-color: transparent;
  }
  .p5-3 {
    grid-area: p5-3;
    background-color: var(--local-button-color-1);
  }
  .p5-4 {
    grid-area: p5-4;
    background-color: transparent;
  }
  .p5-5 {
    grid-area: p5-5;
    background-color: transparent;
  }

  .flat-top > .p1-3,
  .flat-top > .p2-2,
  .flat-top > .p2-4 {
    background-color: transparent;
  }

  .flat-top > .p1-1,
  .flat-top > .p1-5,
  .flat-top > .p2-1,
  .flat-top > .p2-5 {
    background-color: var(--local-button-color-1);
  }

  .flat-bottom > .p4-2,
  .flat-bottom > .p4-4 {
    background-color: transparent;
  }

  .flat-bottom > .p4-1,
  .flat-bottom > .p4-5,
  .flat-bottom > .p5-3,
  .flat-bottom > .p5-1,
  .flat-bottom > .p5-2,
  .flat-bottom > .p5-4,
  .flat-bottom > .p5-5 {
    background-color: var(--local-button-color-1);
  }
</style>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  style:width={styleWidth}
  style:cursor={styleCursor}
  class="container"
  class:flat-top={flatTop}
  class:flat-bottom={flatBottom}
  {onclick}
  role="button"
  tabindex={onclick !== undefined ? 0 : -1}
  {style}
  style:min-height={styleMinHeight}
  style:height={styleHeight}>
  <div class="p1-1"></div>
  <div class="p1-2"></div>
  <div class="p1-3"></div>
  <div class="p1-4"></div>
  <div class="p1-5"></div>

  <div class="p2-1"></div>
  <div class="p2-2"></div>
  <div class="p2-3"></div>
  <div class="p2-4"></div>
  <div class="p2-5"></div>

  <div class="p3-1"></div>
  <div class="p3-2"></div>
  <div
    class="p3-3"
    style:min-width={styleMinWidth}
    style:display={styleDisplay}
    style:position={stylePosition}
    style:padding={stylePadding}
    style:gap={styleGap}
    style:overflow={styleOverflow}
    style:justify-content={styleJustifyContent}
    style:align-items={styleAlignItems}
    style:flex-direction={styleFlexDirection}>
    {@render children()}
  </div>
  <div class="p3-4"></div>
  <div class="p3-5"></div>

  <div class="p4-1"></div>
  <div class="p4-2"></div>
  <div class="p4-3"></div>
  <div class="p4-4"></div>
  <div class="p4-5"></div>

  <div class="p5-1"></div>
  <div class="p5-2"></div>
  <div class="p5-3"></div>
  <div class="p5-4"></div>
  <div class="p5-5"></div>
</div>
