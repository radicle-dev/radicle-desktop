<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    children: Snippet;
    variant: "primary" | "secondary" | "ghost" | "success" | "danger";
    onclick?: () => void;
    disabled?: boolean;
    active?: boolean;
    flatLeft?: boolean;
    flatRight?: boolean;
  }

  const {
    children,
    variant,
    onclick = undefined,
    disabled = false,
    active = false,
    flatLeft = false,
    flatRight = false,
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
  .container {
    white-space: nowrap;

    -webkit-touch-callout: none;
    -webkit-user-select: none;
    user-select: none;

    height: 2rem;
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

  .container:hover:not(.disabled) .p1-3,
  .container:hover:not(.disabled) .p2-2,
  .container:hover:not(.disabled) .p2-4,
  .container:hover:not(.disabled) .p3-1,
  .container:hover:not(.disabled) .p3-3,
  .container:hover:not(.disabled) .p4-2 {
    background-color: var(--button-color-2);
  }

  .container:hover:not(.disabled) .p2-3,
  .container:hover:not(.disabled) .p3-2 {
    background-color: var(--button-color-4);
  }

  .container:hover:not(.disabled) .p3-4,
  .container:hover:not(.disabled) .p3-5,
  .container:hover:not(.disabled) .p4-3,
  .container:hover:not(.disabled) .p4-4,
  .container:hover:not(.disabled) .p5-3 {
    background-color: var(--button-color-1);
  }

  .container.active:not(.disabled) .p1-3,
  .container.active:not(.disabled) .p2-2,
  .container.active:not(.disabled) .p2-4,
  .container.active:not(.disabled) .p3-1,
  .container.active:not(.disabled) .p3-3,
  .container.active:not(.disabled) .p3-5,
  .container.active:not(.disabled) .p4-2,
  .container.active:not(.disabled) .p4-4,
  .container.active:not(.disabled) .p5-3,
  .container:active:not(.disabled) .p1-3,
  .container:active:not(.disabled) .p2-2,
  .container:active:not(.disabled) .p2-4,
  .container:active:not(.disabled) .p3-1,
  .container:active:not(.disabled) .p3-3,
  .container:active:not(.disabled) .p3-5,
  .container:active:not(.disabled) .p4-2,
  .container:active:not(.disabled) .p4-4,
  .container:active:not(.disabled) .p5-3 {
    background-color: var(--button-color-1);
  }

  .container.active:not(.disabled) .p2-3,
  .container.active:not(.disabled) .p3-2,
  .container:active:not(.disabled) .p2-3,
  .container:active:not(.disabled) .p3-2 {
    background-color: var(--button-color-3);
  }

  .container.active:not(.disabled) .p3-4,
  .container.active:not(.disabled) .p4-3,
  .container:active:not(.disabled) .p3-4,
  .container:active:not(.disabled) .p4-3 {
    background-color: var(--button-color-2);
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

  .flat-right .p1-4,
  .flat-right .p1-5,
  .flat-right .p2-5,
  .flat-right .p3-4 {
    background-color: var(--button-color-1);
  }
  .flat-right .p2-4 {
    background-color: var(--button-color-2);
  }
  .flat-right .p4-5,
  .flat-right .p5-4,
  .flat-right .p5-5 {
    background-color: var(--button-color-3);
  }

  .container:hover:not(.disabled).flat-right .p1-4,
  .container:hover:not(.disabled).flat-right .p1-5,
  .container:hover:not(.disabled).flat-right .p2-5,
  .container:hover:not(.disabled).flat-right .p3-4 {
    background-color: var(--button-color-2);
  }
  .container:hover:not(.disabled).flat-right .p2-4 {
    background-color: var(--button-color-4);
  }
  .container:hover:not(.disabled).flat-right .p4-5,
  .container:hover:not(.disabled).flat-right .p5-4,
  .container:hover:not(.disabled).flat-right .p5-5 {
    background-color: var(--button-color-1);
  }

  .container.active:not(.disabled).flat-right .p1-4,
  .container.active:not(.disabled).flat-right .p1-5,
  .container.active:not(.disabled).flat-right .p2-5,
  .container.active:not(.disabled).flat-right .p3-4,
  .container.active:not(.disabled).flat-right .p4-5,
  .container.active:not(.disabled).flat-right .p5-4,
  .container.active:not(.disabled).flat-right .p5-5,
  .container:active:not(.disabled).flat-right .p1-4,
  .container:active:not(.disabled).flat-right .p1-5,
  .container:active:not(.disabled).flat-right .p2-5,
  .container:active:not(.disabled).flat-right .p3-4,
  .container:active:not(.disabled).flat-right .p4-5,
  .container:active:not(.disabled).flat-right .p5-4,
  .container:active:not(.disabled).flat-right .p5-5 {
    background-color: var(--button-color-1);
  }
  .container.active:not(.disabled).flat-right .p2-4,
  .container:active:not(.disabled).flat-right .p2-4 {
    background-color: var(--button-color-3);
  }
  .container.active:not(.disabled).flat-right .p3-5,
  .container.active:not(.disabled).flat-right .p4-4,
  .container:active:not(.disabled).flat-right .p3-5,
  .container:active:not(.disabled).flat-right .p4-4 {
    background-color: var(--button-color-2);
  }

  .flat-left .p1-1,
  .flat-left .p1-2,
  .flat-left .p2-1,
  .flat-left .p3-2 {
    background-color: var(--button-color-1);
  }
  .flat-left .p2-2,
  .flat-left .p3-1 {
    background-color: var(--button-color-2);
  }
  .flat-left .p4-1,
  .flat-left .p4-2,
  .flat-left .p5-1,
  .flat-left .p5-2 {
    background-color: var(--button-color-3);
  }

  .container:hover:not(.disabled).flat-left .p1-1,
  .container:hover:not(.disabled).flat-left .p1-2,
  .container:hover:not(.disabled).flat-left .p2-1,
  .container:hover:not(.disabled).flat-left .p3-2 {
    background-color: var(--button-color-2);
  }
  .container:hover:not(.disabled).flat-left .p2-2,
  .container:hover:not(.disabled).flat-left .p3-1 {
    background-color: var(--button-color-4);
  }
  .container:hover:not(.disabled).flat-left .p4-1,
  .container:hover:not(.disabled).flat-left .p4-2,
  .container:hover:not(.disabled).flat-left .p5-1,
  .container:hover:not(.disabled).flat-left .p5-2 {
    background-color: var(--button-color-1);
  }

  .container.active:not(.disabled).flat-left .p1-1,
  .container.active:not(.disabled).flat-left .p1-2,
  .container.active:not(.disabled).flat-left .p2-1,
  .container.active:not(.disabled).flat-left .p3-2,
  .container.active:not(.disabled).flat-left .p4-1,
  .container.active:not(.disabled).flat-left .p4-2,
  .container.active:not(.disabled).flat-left .p5-1,
  .container.active:not(.disabled).flat-left .p5-2,
  .container:active:not(.disabled).flat-left .p1-1,
  .container:active:not(.disabled).flat-left .p1-2,
  .container:active:not(.disabled).flat-left .p2-1,
  .container:active:not(.disabled).flat-left .p3-2,
  .container:active:not(.disabled).flat-left .p4-1,
  .container:active:not(.disabled).flat-left .p4-2,
  .container:active:not(.disabled).flat-left .p5-1,
  .container:active:not(.disabled).flat-left .p5-2 {
    background-color: var(--button-color-1);
  }
  .container.active:not(.disabled).flat-left .p2-2,
  .container.active:not(.disabled).flat-left .p3-1,
  .container:active:not(.disabled).flat-left .p2-2,
  .container:active:not(.disabled).flat-left .p3-1 {
    background-color: var(--button-color-3);
  }
  .container.active:not(.disabled).flat-left .p4-2,
  .container:active:not(.disabled).flat-left .p4-2 {
    background-color: var(--button-color-2);
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
    background-color: var(--button-color-1);
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
    background-color: var(--button-color-1);
  }
  .p2-3 {
    grid-area: p2-3;
    background-color: var(--button-color-2);
  }
  .p2-4 {
    grid-area: p2-4;
    background-color: var(--button-color-1);
  }
  .p2-5 {
    grid-area: p2-5;
    background-color: transparent;
  }

  .p3-1 {
    grid-area: p3-1;
    background-color: var(--button-color-1);
  }
  .p3-2 {
    grid-area: p3-2;
    background-color: var(--button-color-2);
  }
  .p3-3 {
    grid-area: p3-3;
    background-color: var(--button-color-1);
    padding: 0 8px;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  .p3-4 {
    grid-area: p3-4;
    background-color: var(--button-color-3);
  }
  .p3-5 {
    grid-area: p3-5;
    background-color: var(--button-color-3);
  }

  .p4-1 {
    grid-area: p4-1;
    background-color: transparent;
  }
  .p4-2 {
    grid-area: p4-2;
    background-color: var(--button-color-1);
  }
  .p4-3 {
    grid-area: p4-3;
    background-color: var(--button-color-3);
  }
  .p4-4 {
    grid-area: p4-4;
    background-color: var(--button-color-3);
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
    background-color: var(--button-color-3);
  }
  .p5-4 {
    grid-area: p5-4;
    background-color: transparent;
  }
  .p5-5 {
    grid-area: p5-5;
    background-color: transparent;
  }
</style>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  class="container active"
  style:cursor={!disabled ? "pointer" : "default"}
  class:disabled
  class:active
  class:flat-right={flatRight}
  class:flat-left={flatLeft}
  onclick={!disabled ? onclick : undefined}
  role="button"
  tabindex="0"
  {style}>
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
  <div class="p3-3 txt-semibold txt-small">
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
