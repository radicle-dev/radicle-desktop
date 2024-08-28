<script lang="ts">
  export let variant: "primary" | "secondary" | "ghost" | "transparent";
  export let stylePadding: string | undefined = undefined;

  $: style =
    variant === "transparent"
      ? "--button-color-1: transparent"
      : `--button-color-1: var(--color-fill-${variant});`;
</script>

<style>
  .pixel {
    background-color: var(--button-color-1);
  }

  .p1-1 {
    grid-area: p1-1;
    background-color: transparent;
  }
  .p1-2 {
    grid-area: p1-2;
  }
  .p1-3 {
    grid-area: p1-3;
    background-color: transparent;
  }

  .p2-1 {
    grid-area: p2-1;
  }
  .p2-2 {
    grid-area: p2-2;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  .p2-3 {
    grid-area: p2-3;
  }

  .p3-1 {
    grid-area: p3-1;
    background-color: transparent;
  }
  .p3-2 {
    grid-area: p3-2;
  }
  .p3-3 {
    grid-area: p3-3;
    background-color: transparent;
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
    grid-template-columns: 2px auto 2px;
    grid-template-rows: 2px auto 2px;
    grid-template-areas:
      "p1-1 p1-2 p1-3"
      "p2-1 p2-2 p2-3"
      "p3-1 p3-2 p3-3";
  }
</style>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<div class="container" on:click role="button" tabindex="0" {style}>
  <div class="pixel p1-1" />
  <div class="pixel p1-2" />
  <div class="pixel p1-3" />

  <div class="pixel p2-1" />
  <div class="pixel p2-2 txt-semibold txt-small" style:padding={stylePadding}>
    <slot />
  </div>
  <div class="pixel p2-3" />

  <div class="pixel p3-1" />
  <div class="pixel p3-2" />
  <div class="pixel p3-3" />
</div>
