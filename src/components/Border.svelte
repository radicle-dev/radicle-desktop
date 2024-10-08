<script lang="ts">
  export let variant: "primary" | "secondary" | "ghost";
  export let hoverable: boolean = false;
  export let onclick: (() => void) | undefined = undefined;

  export let stylePadding: string | undefined = undefined;
  export let styleHeight: string | undefined = undefined;
  export let styleMinHeight: string | undefined = undefined;
  export let styleWidth: string | undefined = undefined;
  export let styleCursor: "default" | "pointer" = "default";
  export let styleGap: string = "0.5rem";

  $: style =
    `--local-button-color-1: var(--color-fill-${variant});` +
    `--local-hover-background-color: ${hoverable ? "var(--color-background-float)" : "var(--color-background-default)"}`;
</script>

<style>
  .pixel {
    background-color: transparent;
  }

  .p1-1 {
    grid-area: p1-1;
  }
  .p1-2 {
    grid-area: p1-2;
  }
  .p1-3 {
    grid-area: p1-3;
    background-color: var(--local-button-color-1);
  }
  .p1-4 {
    grid-area: p1-4;
  }
  .p1-5 {
    grid-area: p1-5;
  }

  .p2-1 {
    grid-area: p2-1;
  }
  .p2-2 {
    grid-area: p2-2;
    background-color: var(--local-button-color-1);
  }
  .p2-3 {
    grid-area: p2-3;
  }
  .p2-4 {
    grid-area: p2-4;
    background-color: var(--local-button-color-1);
  }
  .p2-5 {
    grid-area: p2-5;
  }

  .p3-1 {
    grid-area: p3-1;
    background-color: var(--local-button-color-1);
  }
  .p3-2 {
    grid-area: p3-2;
  }
  .p3-3 {
    grid-area: p3-3;
    display: flex;
    align-items: center;
  }
  .p3-4 {
    grid-area: p3-4;
  }
  .p3-5 {
    grid-area: p3-5;
    background-color: var(--local-button-color-1);
  }

  .p4-1 {
    grid-area: p4-1;
  }
  .p4-2 {
    grid-area: p4-2;
    background-color: var(--local-button-color-1);
  }
  .p4-3 {
    grid-area: p4-3;
  }
  .p4-4 {
    grid-area: p4-4;
    background-color: var(--local-button-color-1);
  }
  .p4-5 {
    grid-area: p4-5;
  }

  .p5-1 {
    grid-area: p5-1;
  }
  .p5-2 {
    grid-area: p5-2;
  }
  .p5-3 {
    grid-area: p5-3;
    background-color: var(--local-button-color-1);
  }
  .p5-4 {
    grid-area: p5-4;
  }
  .p5-5 {
    grid-area: p5-5;
  }

  .container {
    white-space: nowrap;

    -webkit-touch-callout: none;
    -webkit-user-select: none;
    user-select: none;

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
    overflow: hidden;
  }
  .container .p2-3,
  .container .p3-2,
  .container .p3-3,
  .container .p3-4,
  .container .p4-3 {
    background-color: var(--color-background-default);
  }

  .container:hover .p2-3,
  .container:hover .p3-2,
  .container:hover .p3-3,
  .container:hover .p3-4,
  .container:hover .p4-3 {
    background-color: var(--local-hover-background-color);
  }
</style>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<div
  style:width={styleWidth}
  style:cursor={styleCursor}
  class="container"
  {onclick}
  role="button"
  tabindex={onclick !== undefined ? 0 : -1}
  {style}
  style:min-height={styleMinHeight}
  style:height={styleHeight}>
  <div class="pixel p1-1"></div>
  <div class="pixel p1-2"></div>
  <div class="pixel p1-3"></div>
  <div class="pixel p1-4"></div>
  <div class="pixel p1-5"></div>

  <div class="pixel p2-1"></div>
  <div class="pixel p2-2"></div>
  <div class="pixel p2-3"></div>
  <div class="pixel p2-4"></div>
  <div class="pixel p2-5"></div>

  <div class="pixel p3-1"></div>
  <div class="pixel p3-2"></div>
  <div class="pixel p3-3" style:padding={stylePadding} style:gap={styleGap}>
    <slot />
  </div>
  <div class="pixel p3-4"></div>
  <div class="pixel p3-5"></div>

  <div class="pixel p4-1"></div>
  <div class="pixel p4-2"></div>
  <div class="pixel p4-3"></div>
  <div class="pixel p4-4"></div>
  <div class="pixel p4-5"></div>

  <div class="pixel p5-1"></div>
  <div class="pixel p5-2"></div>
  <div class="pixel p5-3"></div>
  <div class="pixel p5-4"></div>
  <div class="pixel p5-5"></div>
</div>
