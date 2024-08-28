<script lang="ts">
  import { unreachable } from "@app/lib/utils";

  export let name:
    | "dashboard"
    | "arrow-left"
    | "arrow-right"
    | "plus"
    | "offline"
    | "chevron-right"
    | "inbox"
    | "delegate"
    | "diff"
    | "file"
    | "repo"
    | "seedling";
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
  {:else if name === "chevron-right"}
    <path d="M9 7L10 7L10 8L9 8L9 7Z" />
    <path d="M8 6H9V7H8L8 6Z" />
    <path d="M7 5L8 5V6L7 6L7 5Z" />
    <path d="M6 4L7 4L7 5L6 5L6 4Z" />
    <path d="M5 3L6 3L6 4H5L5 3Z" />
    <path d="M5 12H6L6 13H5L5 12Z" />
    <path d="M6 11H7L7 12H6L6 11Z" />
    <path d="M7 10L8 10L8 11L7 11L7 10Z" />
    <path d="M9 8L10 8V9L9 9V8Z" />
    <path d="M8 9H9V10H8L8 9Z" />
  {:else if name === "inbox"}
    <path d="M2 3H3V13H2V3Z" />
    <path d="M13 3H14V13H13V3Z" />
    <path d="M3 7H5V8H3V7Z" />
    <path d="M11 7H14V8H11V7Z" />
    <path d="M5 8H6V9H5L5 8Z" />
    <path d="M10 8H11V9H10V8Z" />
    <path d="M6 9H10L10 10H6L6 9Z" />
    <path d="M3 13H13V14H3L3 13Z" />
    <path d="M3 2H13V3H3L3 2Z" />
  {:else if name === "repo"}
    <path d="M13 5H14V7H13V5Z" />
    <path d="M13 10H14V11H13V10Z" />
    <path d="M13 7H14V5H13V7Z" />
    <path d="M13 9H14V8H13V9Z" />
    <path d="M3 7H2L2 5H3L3 7Z" />
    <path d="M3 8H2L2 9H3L3 8Z" />
    <path d="M3 5H2L2 7H3L3 5Z" />
    <path d="M3 10L2 10L2 11H3V10Z" />
    <path d="M11 4H13V5L11 5V4Z" />
    <path d="M11 8L13 8V7H11V8Z" />
    <path d="M11 10H13V9H11V10Z" />
    <path d="M11 12H13V11H11V12Z" />
    <path d="M9 3H11V4L9 4V3Z" />
    <path d="M9 9L11 9V8H9V9Z" />
    <path d="M9 11H11V10H9V11Z" />
    <path d="M9 13H11V12H9V13Z" />
    <path d="M8 2H9V3L8 3V2Z" />
    <path d="M8 10L9 10V9H8V10Z" />
    <path d="M8 12L9 12V11L8 11V12Z" />
    <path d="M8 14H9V13H8V14Z" />
    <path d="M7 2L8 2V3L7 3V2Z" />
    <path d="M7 10L8 10V9L7 9V10Z" />
    <path d="M7 12L8 12V11L7 11V12Z" />
    <path d="M7 14H8V13H7V14Z" />
    <path d="M5 5L3 5L3 4L5 4V5Z" />
    <path d="M5 7H3L3 8L5 8L5 7Z" />
    <path d="M5 9H3L3 10H5L5 9Z" />
    <path d="M5 11H3L3 12H5V11Z" />
    <path d="M7 4L5 4V3H7L7 4Z" />
    <path d="M7 8H5V9L7 9L7 8Z" />
    <path d="M7 10H5L5 11H7L7 10Z" />
    <path d="M7 12H5L5 13H7L7 12Z" />
  {:else if name === "dashboard"}
    <path d="M2 11H14V12H2V11Z" />
    <path d="M2 9H3V11H2L2 9Z" />
    <path d="M14 9H13V11L14 11V9Z" />
    <path d="M10 4V5L6 5V4L10 4Z" />
    <path d="M3 7H4V9L3 9L3 7Z" />
    <path d="M13 7H12V9H13V7Z" />
    <path d="M4 6H5V7H4L4 6Z" />
    <path d="M12 6H11V7H12V6Z" />
    <path d="M5 5L6 5V6H5V5Z" />
    <path d="M11 5L10 5V6H11V5Z" />
    <path d="M7 8H8V10H7V8Z" />
    <path d="M8 6H9V8H8V6Z" />
    <path d="M6 10H10V11H6V10Z" />
  {:else if name === "diff"}
    <path d="M2 3H3V13H2V3Z" />
    <path d="M3 13H12V14H3L3 13Z" />
    <path d="M3 2H12V3L3 3L3 2Z" />
    <path d="M12 3L13 3V13H12V3Z" />
    <path d="M7 4H8V9H7V4Z" />
    <path d="M5 6H10V7H5V6Z" />
    <path d="M5 10H10V11H5V10Z" />
  {:else if name === "file"}
    <path d="M10 4H11V5H10V4Z" />
    <path d="M11 5L12 5V6H11V5Z" />
    <path d="M10 6L12 6V7H10V6Z" />
    <path d="M9 6H10V7H9V6Z" />
    <path d="M8 5H9L9 6L8 6V5Z" />
    <path d="M8 3H9V5H8V3Z" />
    <path d="M9 3L10 3V4H9V3Z" />
    <path d="M4 13H12V14H4V13Z" />
    <path d="M4 2H9V3L4 3V2Z" />
    <path d="M13 6V13H12V6L13 6Z" />
    <path d="M4 3V13H3L3 3L4 3Z" />
  {:else if name === "delegate"}
    <path d="M2.33301 9L2.33301 7H3.33301V9H2.33301Z" />
    <path d="M3.33301 11L3.33301 9H4.33301L4.33301 11H3.33301Z" />
    <path d="M4.33301 12L4.33301 11H7.33301V12H4.33301Z" />
    <path d="M7.33301 11L7.33301 10H8.33301V11H7.33301Z" />
    <path d="M7.33301 6V5H8.33301V6H7.33301Z" />
    <path d="M4.33301 5V4L7.33301 4V5L4.33301 5Z" />
    <path d="M3.33301 7L3.33301 5L4.33301 5L4.33301 7H3.33301Z" />
    <path d="M4.33301 9L4.33301 7L5.33301 7V9L4.33301 9Z" />
    <path d="M7.33301 9V7H8.33301V9H7.33301Z" />
    <path d="M8.33301 7L8.33301 6L13.333 6V7L8.33301 7Z" />
    <path d="M8.33301 10L8.33301 9L10.333 9V10L8.33301 10Z" />
    <path d="M13.333 9V7L14.333 7V9H13.333Z" />
    <path d="M10.333 8H11.333V9L10.333 9V8Z" />
    <path d="M11.333 9L13.333 9V10H11.333L11.333 9Z" />
    <path d="M5.33301 6L7.33301 6L7.33301 7L5.33301 7L5.33301 6Z" />
    <path d="M5.33301 9L7.33301 9L7.33301 10H5.33301V9Z" />
  {:else if name === "seedling"}
    <path d="M10.333 6H9.33301V5L10.333 5V6Z" />
    <path d="M11.333 5L10.333 5V4L11.333 4V5Z" />
    <path d="M12.333 4H11.333L11.333 3L12.333 3L12.333 4Z" />
    <path d="M13.333 4L12.333 4L12.333 3L13.333 3L13.333 4Z" />
    <path d="M14.333 5L13.333 5V4L14.333 4V5Z" />
    <path d="M13.333 6H12.333V5H13.333V6Z" />
    <path d="M5.33301 7L5.33301 6L4.33301 6L4.33301 7L5.33301 7Z" />
    <path d="M4.33301 8V7L3.33301 7L3.33301 8L4.33301 8Z" />
    <path d="M3.33301 9L3.33301 8L2.33301 8L2.33301 9H3.33301Z" />
    <path d="M3.33301 10L3.33301 9H2.33301L2.33301 10H3.33301Z" />
    <path d="M4.33301 11L4.33301 10H3.33301L3.33301 11L4.33301 11Z" />
    <path d="M10.333 7H9.33301L9.33301 6H10.333L10.333 7Z" />
    <path d="M9.33301 8H8.33301V7L9.33301 7L9.33301 8Z" />
    <path d="M8.33301 8H7.33301V7H8.33301V8Z" />
    <path d="M11.333 8L10.333 8L10.333 7L11.333 7V8Z" />
    <path d="M12.333 8L10.333 8L10.333 7L12.333 7V8Z" />
    <path d="M13.333 7H12.333V6H13.333V7Z" />
    <path d="M5.33301 10L5.33301 9H4.33301L4.33301 10L5.33301 10Z" />
    <path d="M9.33301 8H8.33301V11H9.33301V8Z" />
    <path d="M8.33301 11L7.33301 11V14H8.33301V11Z" />
    <path d="M6.33301 7L6.33301 6H5.33301L5.33301 7H6.33301Z" />
    <path d="M7.33301 8V7L6.33301 7L6.33301 8H7.33301Z" />
    <path d="M7.33301 9L7.33301 7L6.33301 7L6.33301 9H7.33301Z" />
    <path d="M6.33301 10V9H5.33301L5.33301 10H6.33301Z" />
  {:else}
    {unreachable(name)}
  {/if}
</svg>
