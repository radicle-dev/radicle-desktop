<script lang="ts">
  import { unreachable } from "@app/lib/utils";
  export let name: "arrow-left" | "arrow-right" | "plus" | "offline";
</script>

<style>
  svg {
    display: flex;
    flex-shrink: 0;
  }
</style>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
<svg
  role="img"
  on:click
  width="16"
  height="16"
  fill="currentColor"
  viewBox="0 0 16 16">
  {#if name === "arrow-left"}
    <path d="M1.99997 7L1.99997 8L2.99997 8L2.99997 7L1.99997 7Z" />
    <path d="M2.99997 6L2.99997 7L3.99997 7L3.99997 6L2.99997 6Z" />
    <path d="M2.99997 9L2.99997 8L3.99997 8L3.99997 9L2.99997 9Z" />
    <path d="M2.99997 9L14 9L14 7L2.99997 7L2.99997 9Z" />
    <path d="M3.99997 11L4.99997 11L4.99997 5L3.99997 5L3.99997 11Z" />
    <path d="M4.99997 12L5.99997 12L5.99997 4L4.99997 4L4.99997 12Z" />
    <path d="M5.99997 13L6.99997 13L6.99997 3L5.99997 3L5.99997 13Z" />
    <path d="M2.99997 10L3.99997 10L3.99997 9L2.99997 9L2.99997 10Z" />
    <path d="M1.99997 9L2.99997 9L2.99997 8L1.99997 8L1.99997 9Z" />
  {:else if name === "arrow-right"}
    <path d="M14 9V8H13V9H14Z" />
    <path d="M13 10V9H12V10H13Z" />
    <path d="M13 7V8H12V7H13Z" />
    <path d="M13 7H2.00003V9H13V7Z" />
    <path d="M12 5H11V11H12V5Z" />
    <path d="M11 4H10V12H11V4Z" />
    <path d="M10 3H9.00003V13H10V3Z" />
    <path d="M13 6H12V7H13V6Z" />
    <path d="M14 7H13V8H14V7Z" />
  {:else if name === "plus"}
    <path d="M7.00002 2H9.00002V14H7.00002V2Z" />
    <path d="M14 7V9L2.00002 9L2.00002 7L14 7Z" />
  {:else if name === "offline"}
    <path d="M3 6L3 8H2L2 6H3Z" />
    <path d="M13 10V8H14V10H13Z" />
    <path d="M5 9V7H4V9H5Z" />
    <path d="M11 7V9H12V7H11Z" />
    <path d="M4 4L4 6H3L3 4H4Z" />
    <path d="M12 12V10H13V12H12Z" />
    <path d="M4 10V12H3L3 10H4Z" />
    <path d="M12 6V5H13V6H12Z" />
    <path d="M5 4H4V3L5 3V4Z" />
    <path d="M11 12H12V13H11V12Z" />
    <path d="M3 8L3 10H2L2 8H3Z" />
    <path d="M13 8V6H14V8H13Z" />
    <path d="M10 5L11 5V6L10 6V5Z" />
    <path d="M6 7H5L5 5H6V7Z" />
    <path d="M10 9L11 9V11H10V9Z" />
    <path d="M7 7H9V8H7V7Z" />
    <path d="M6 11H7V12H6V11Z" />
    <path d="M10 5H9V4H10V5Z" />
    <path d="M6 4H7V5H6L6 4Z" />
    <path d="M10 12H9V11H10V12Z" />
    <path d="M5 2H6V3L5 3L5 2Z" />
    <path d="M11 14H10V13H11V14Z" />
    <path d="M5 13H6V14H5V13Z" />
    <path d="M11 3L10 3V2L11 2V3Z" />
    <path d="M12 4H11V3L12 3V4Z" />
    <path d="M13 2.00001H14V3.00001H13V2.00001Z" />
    <path d="M12 3L13 3.00001V4.00001L12 4V3Z" />
    <path d="M11 4H12V5L11 5V4Z" />
    <path d="M10 5L11 5V6L10 6V5Z" />
    <path d="M9 6.00001L10 6L10 7.00001L9 7L9 6.00001Z" />
    <path d="M8 7.00001L9 7V8L8 8.00001V7.00001Z" />
    <path d="M7 8L8 8.00001V9.00001H7L7 8Z" />
    <path d="M6 9L7 9.00001V10H6V9Z" />
    <path d="M5 10H6V11L5 11V10Z" />
    <path d="M4 11L5 11V12L4 12L4 11Z" />
    <path d="M3 12H4L4 13H3L3 12Z" />
    <path d="M2 13L3 13L3 14H2V13Z" />
  {:else}
    {unreachable(name)}
  {/if}
</svg>
