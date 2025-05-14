<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    popoverToggle?: string;
    active?: boolean;
    children: Snippet;
    disabled?: boolean;
    onclick?: () => void;
    styleHeight?: "2rem" | "2.5rem";
    title?: string;
    variant: "primary" | "secondary" | "ghost";
  }

  const {
    popoverToggle,
    active = false,
    children,
    disabled = false,
    onclick,
    styleHeight = "2rem",
    title,
    variant,
  }: Props = $props();

  const style = $derived(
    `--button-color-1: var(--color-fill-${variant});` +
      `--button-color-2: var(--color-fill-${variant}-hover);` +
      `--button-color-3: var(--color-fill-${variant}-shade);` +
      // The ghost colors are called --color-fill-counter and --color-fill-counter-emphasized.
      `--button-color-4: var(--color-fill${variant === "ghost" ? "" : `-${variant}`}-counter)`,
  );
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
    background-color: var(--button-color-1);
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
    background-color: var(--button-color-1);
  }
  .p2-3 {
    grid-area: p2-3;
  }
  .p2-4 {
    grid-area: p2-4;
    background-color: var(--button-color-1);
  }
  .p2-5 {
    grid-area: p2-5;
  }

  .p3-1 {
    grid-area: p3-1;
    background-color: var(--button-color-1);
  }
  .p3-2 {
    grid-area: p3-2;
  }
  .p3-3 {
    grid-area: p3-3;
    padding: 0 0.5rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  .p3-4 {
    grid-area: p3-4;
  }
  .p3-5 {
    grid-area: p3-5;
    background-color: var(--button-color-1);
  }

  .p4-1 {
    grid-area: p4-1;
  }
  .p4-2 {
    grid-area: p4-2;
    background-color: var(--button-color-1);
  }
  .p4-3 {
    grid-area: p4-3;
  }
  .p4-4 {
    grid-area: p4-4;
    background-color: var(--button-color-1);
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
    background-color: var(--button-color-1);
  }
  .p5-4 {
    grid-area: p5-4;
  }
  .p5-5 {
    grid-area: p5-5;
  }

  .container:hover:not(.disabled) .p1-3 {
    background-color: var(--button-color-1);
  }

  .container:hover:not(.disabled) .p2-2 {
    background-color: var(--button-color-1);
  }
  .container:hover:not(.disabled) .p2-3 {
    background-color: var(--button-color-1);
  }
  .container:hover:not(.disabled) .p2-4 {
    background-color: var(--button-color-1);
  }

  .container:hover:not(.disabled) .p3-1 {
    background-color: var(--button-color-1);
  }
  .container:hover:not(.disabled) .p3-2 {
    background-color: var(--button-color-1);
  }
  .container:hover:not(.disabled) .p3-3 {
    background-color: var(--button-color-1);
  }
  .container:hover:not(.disabled) .p3-4 {
    background-color: var(--button-color-1);
  }
  .container:hover:not(.disabled) .p3-5 {
    background-color: var(--button-color-1);
  }

  .container:hover:not(.disabled) .p4-2 {
    background-color: var(--button-color-1);
  }
  .container:hover:not(.disabled) .p4-3 {
    background-color: var(--button-color-1);
  }
  .container:hover:not(.disabled) .p4-4 {
    background-color: var(--button-color-1);
  }

  .container:hover:not(.disabled) .p5-3 {
    background-color: var(--button-color-1);
  }

  .container:active:not(.disabled) .p1-3,
  .container.active:not(.disabled) .p1-3 {
    background-color: var(--button-color-1);
  }

  .container:active:not(.disabled) .p2-2,
  .container.active:not(.disabled) .p2-2 {
    background-color: var(--button-color-1);
  }
  .container:active:not(.disabled) .p2-3,
  .container.active:not(.disabled) .p2-3 {
    background-color: var(--button-color-3);
  }
  .container:active:not(.disabled) .p2-4,
  .container.active:not(.disabled) .p2-4 {
    background-color: var(--button-color-1);
  }

  .container:active:not(.disabled) .p3-1,
  .container.active:not(.disabled) .p3-1 {
    background-color: var(--button-color-1);
  }
  .container:active:not(.disabled) .p3-2,
  .container.active:not(.disabled) .p3-2 {
    background-color: var(--button-color-3);
  }
  .container:active:not(.disabled) .p3-3,
  .container.active:not(.disabled) .p3-3 {
    background-color: var(--button-color-1);
  }
  .container:active:not(.disabled) .p3-4,
  .container.active:not(.disabled) .p3-4 {
    background-color: var(--button-color-2);
  }
  .container:active:not(.disabled) .p3-5,
  .container.active:not(.disabled) .p3-5 {
    background-color: var(--button-color-1);
  }

  .container:active:not(.disabled) .p4-2,
  .container.active:not(.disabled) .p4-2 {
    background-color: var(--button-color-1);
  }
  .container:active:not(.disabled) .p4-3,
  .container.active:not(.disabled) .p4-3 {
    background-color: var(--button-color-2);
  }
  .container:active:not(.disabled) .p4-4,
  .container.active:not(.disabled) .p4-4 {
    background-color: var(--button-color-1);
  }
  .container:active:not(.disabled) .p5-3,
  .container.active:not(.disabled) .p5-3 {
    background-color: var(--button-color-1);
  }

  .container.active:not(.disabled) {
    color: var(--color-foreground-emphasized);
  }

  .container.disabled {
    color: var(--color-foreground-disabled);
  }

  .disabled .p1-3,
  .disabled .p2-2,
  .disabled .p2-3,
  .disabled .p2-4,
  .disabled .p3-1,
  .disabled .p3-2,
  .disabled .p3-3,
  .disabled .p3-4,
  .disabled .p3-5,
  .disabled .p4-2,
  .disabled .p4-3,
  .disabled .p4-4,
  .disabled .p5-3 {
    background-color: var(--color-fill-ghost);
  }

  .container {
    cursor: pointer;
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
  }
</style>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  data-popover-toggle={popoverToggle}
  class:active
  class:disabled
  class="container"
  onclick={!disabled ? onclick : undefined}
  role="button"
  style:cursor={!disabled ? "pointer" : "default"}
  style:height={styleHeight}
  tabindex="0"
  {style}
  {title}>
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
  <div class="pixel p3-3 txt-semibold txt-small">
    {@render children()}
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
