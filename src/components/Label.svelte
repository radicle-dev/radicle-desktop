<script lang="ts">
  export let styleBorderColor: string;
  export let styleFillColor: string = "transparent";
  export let emptyPixels: 2 | 1 = 2;

  let gridTemplate = `${"2px ".repeat(emptyPixels)}auto ${"2px ".repeat(emptyPixels)}`;
</script>

<style>
  .content {
    padding: 2px 8px;
    display: flex;
    gap: 6px;
    align-items: center;
  }
  .container {
    white-space: nowrap;
    user-select: none;
    column-gap: 0;
    height: 2rem;
    row-gap: 0;
    display: grid;
  }
</style>

<div
  class="container"
  style:grid-template-columns={gridTemplate}
  style:grid-template-rows={gridTemplate}>
  {@html "<div></div>".repeat(emptyPixels)}
  <div style:background-color={styleBorderColor} />
  {@html "<div></div>".repeat(emptyPixels)}

  {#if emptyPixels === 2}
    <div />
    <div style:background-color={styleBorderColor} />
    <div style:background-color={styleFillColor} />
    <div style:background-color={styleBorderColor} />
    <div />
  {/if}

  <div style:background-color={styleBorderColor} />
  {#if emptyPixels === 2}
    <div style:background-color={styleFillColor} />
  {/if}
  <div
    class="content txt-semibold txt-small"
    style:background-color={styleFillColor}>
    <slot />
  </div>
  {#if emptyPixels === 2}
    <div style:background-color={styleFillColor} />
  {/if}
  <div style:background-color={styleBorderColor} />

  {#if emptyPixels === 2}
    <div />
    <div style:background-color={styleBorderColor} />
    <div style:background-color={styleFillColor} />
    <div style:background-color={styleBorderColor} />
    <div />
  {/if}

  {@html "<div></div>".repeat(emptyPixels)}
  <div style:background-color={styleBorderColor} />
  {@html "<div></div>".repeat(emptyPixels)}
</div>
