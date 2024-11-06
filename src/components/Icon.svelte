<script lang="ts">
  import { unreachable } from "@app/lib/utils";

  interface Props {
    size?: "16" | "32";
    onclick?: () => void;
    styleCursor?: "default" | "pointer";
    name:
      | "arrow-left"
      | "arrow-right"
      | "checkmark"
      | "chevron-down"
      | "chevron-right"
      | "comment"
      | "copy"
      | "cross"
      | "dashboard"
      | "delegate"
      | "diff"
      | "eye"
      | "face"
      | "file"
      | "inbox"
      | "issue"
      | "lock"
      | "markdown"
      | "moon"
      | "more-vertical"
      | "offline"
      | "online"
      | "patch"
      | "pen"
      | "plus"
      | "reply"
      | "repo"
      | "revision"
      | "seedling"
      | "seedling-filled"
      | "settings"
      | "sidebar"
      | "sun"
      | "warning";
  }

  const {
    size = "16",
    onclick = undefined,
    styleCursor = "default",
    name,
  }: Props = $props();
</script>

<style>
  svg {
    display: flex;
    flex-shrink: 0;
    -webkit-touch-callout: none;
    -webkit-user-select: none;
    user-select: none;
  }
</style>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<svg
  style:cursor={styleCursor}
  role="img"
  {onclick}
  aria-label={`icon-${name}`}
  width={size}
  height={size}
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
  {:else if name === "checkmark"}
    <path d="M7 11V12H6V11H7Z" />
    <path d="M8 10V11L7 11L7 10H8Z" />
    <path d="M9 9V10H8V9L9 9Z" />
    <path d="M10 8V9L9 9V8H10Z" />
    <path d="M11 7V8L10 8L10 7H11Z" />
    <path d="M12 6V7H11V6H12Z" />
    <path d="M13 5V6L12 6V5L13 5Z" />
    <path d="M4 8V9H3L3 8H4Z" />
    <path d="M5 9L5 10L4 10L4 9H5Z" />
    <path d="M6 10L6 11H5L5 10L6 10Z" />
  {:else if name === "chevron-down"}
    <path d="M9 10V11H8V10H9Z" />
    <path d="M10 9V10L9 10V9H10Z" />
    <path d="M11 8V9H10V8L11 8Z" />
    <path d="M12 7V8L11 8V7H12Z" />
    <path d="M4 6V7H3L3 6L4 6Z" />
    <path d="M13 6V7L12 7L12 6L13 6Z" />
    <path d="M5 7L5 8H4L4 7L5 7Z" />
    <path d="M6 8V9H5L5 8L6 8Z" />
    <path d="M8 10V11H7L7 10H8Z" />
    <path d="M7 9L7 10H6L6 9L7 9Z" />
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
  {:else if name === "comment"}
    <path d="M4 2H12V3H4V2Z" />
    <path d="M12 3L13 3V4H12V3Z" />
    <path d="M12 10H13V11H12V10Z" />
    <path d="M3 3L4 3V4H3V3Z" />
    <path d="M5 11H12V12H5V11Z" />
    <path d="M3 13H4V14H3V13Z" />
    <path d="M4 12H5V13H4L4 12Z" />
    <path d="M5 11H6V12H5V11Z" />
    <path d="M13 4H14V10H13V4Z" />
    <path d="M2 4L3 4L3 13H2V4Z" />
    <path d="M5 5H11V6H5V5Z" />
    <path d="M5 8H11V9H5V8Z" />
  {:else if name === "copy"}
    <path d="M6.5 2H13.5V3H6.5V2Z" />
    <path d="M3.5 5H4.5V6H3.5V5Z" />
    <path d="M7.5 5H10.5V6H7.5V5Z" />
    <path d="M6.5 10H13.5V11H6.5V10Z" />
    <path d="M3.5 13H10.5V14H3.5V13Z" />
    <path d="M13.5 3L14.5 3V10L13.5 10L13.5 3Z" />
    <path d="M5.5 3L6.5 3L6.5 10L5.5 10V3Z" />
    <path d="M2.5 6H3.5L3.5 13H2.5V6Z" />
    <path d="M10.5 12H11.5V13H10.5V12Z" />
    <path d="M10.5 6L11.5 6V9H10.5L10.5 6Z" />
  {:else if name === "cross"}
    <path d="M5.00003 11V12H4.00003L4.00003 11H5.00003Z" />
    <path d="M6.00003 10L6.00003 11H5.00003L5.00003 10H6.00003Z" />
    <path d="M7.00003 9V10H6.00003L6.00003 9L7.00003 9Z" />
    <path d="M8.00003 7V8H7.00003V7H8.00003Z" />
    <path d="M10 6V7H9.00003V6H10Z" />
    <path d="M11 5V6L10 6V5H11Z" />
    <path d="M12 4V5L11 5V4L12 4Z" />
    <path d="M10 10V11H11V10H10Z" />
    <path d="M11 11V12H12V11H11Z" />
    <path d="M9.00003 9V10H10V9H9.00003Z" />
    <path d="M8.00003 8L8.00003 9H9.00003V8L8.00003 8Z" />
    <path d="M8.00003 7V8L9.00003 8V7L8.00003 7Z" />
    <path d="M7.00003 7L7.00003 9H8.00003L8.00003 7H7.00003Z" />
    <path d="M6.00003 6L6.00003 7H7.00003L7.00003 6H6.00003Z" />
    <path d="M5.00003 5V6L6.00003 6V5L5.00003 5Z" />
    <path d="M4.00003 4L4.00003 5L5.00003 5L5.00003 4L4.00003 4Z" />
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
  {:else if name === "diff"}
    <path d="M2 3H3V13H2V3Z" />
    <path d="M3 13H12V14H3L3 13Z" />
    <path d="M3 2H12V3L3 3L3 2Z" />
    <path d="M12 3L13 3V13H12V3Z" />
    <path d="M7 4H8V9H7V4Z" />
    <path d="M5 6H10V7H5V6Z" />
    <path d="M5 10H10V11H5V10Z" />
  {:else if name === "eye"}
    <path d="M10 5L8.00002 5V4L10 4V5Z" />
    <path d="M6.00002 11L8.00002 11L8.00002 12H6.00002L6.00002 11Z" />
    <path d="M7.00002 7L9.00002 7V6L7.00002 6V7Z" />
    <path d="M9.00001 9L7.00001 9V10L9.00001 10V9Z" />
    <path d="M4.00002 6V7H3.00002L3.00002 6H4.00002Z" />
    <path d="M12 10V9H13V10H12Z" />
    <path d="M12 6L10 6V5L12 5V6Z" />
    <path d="M4.00002 10L6.00002 10V11H4.00002L4.00002 10Z" />
    <path d="M6.00002 6L4.00002 6L4.00002 5L6.00002 5V6Z" />
    <path d="M10 10H12V11L10 11L10 10Z" />
    <path d="M12 7V6H13V7H12Z" />
    <path d="M4.00002 9V10H3.00002L3.00002 9H4.00002Z" />
    <path d="M8.00002 5L6.00002 5V4L8.00002 4V5Z" />
    <path d="M8.00002 11L10 11V12H8.00002L8.00002 11Z" />
    <path d="M6.00002 8V7L7.00002 7V8L6.00002 8Z" />
    <path d="M10 8V9H9.00001L9.00002 8H10Z" />
    <path d="M9.00002 8L9.00002 7L10 7V8H9.00002Z" />
    <path d="M7.00002 8L7.00001 9L6.00002 9V8L7.00002 8Z" />
    <path d="M14 7V8H13V7L14 7Z" />
    <path d="M2.00002 9L2.00002 8H3.00002V9L2.00002 9Z" />
    <path d="M3.00002 7L3.00002 8H2.00002L2.00002 7H3.00002Z" />
    <path d="M13 9V8H14V9L13 9Z" />
  {:else if name === "face"}
    <path d="M6 13H8V14H6V13Z" />
    <path d="M10 13L8 13V14L10 14V13Z" />
    <path d="M3 6L3 8H2L2 6H3Z" />
    <path d="M13 6V8H14V6H13Z" />
    <path d="M4 12H6V13L4 13V12Z" />
    <path d="M12 12H10L10 13H12V12Z" />
    <path d="M4 4V6H3L3 4H4Z" />
    <path d="M12 4V6L13 6V4H12Z" />
    <path d="M4 10L4 12H3L3 10H4Z" />
    <path d="M12 10V12L13 12V10H12Z" />
    <path d="M6 4L4 4L4 3L6 3V4Z" />
    <path d="M10 4L12 4V3L10 3V4Z" />
    <path d="M3 8L3 10H2L2 8H3Z" />
    <path d="M13 8V10L14 10V8H13Z" />
    <path d="M8 3L6 3V2L8 2V3Z" />
    <path d="M8 3L10 3L10 2L8 2V3Z" />
    <path d="M9 6H11V7H9V6Z" />
    <path d="M5 9H6V10H5V9Z" />
    <path d="M10 9H11V10H10V9Z" />
    <path d="M6 10H7V11H6L6 10Z" />
    <path d="M7 10L10 10L10 11H7V10Z" />
    <path d="M5 6H7V7H5V6Z" />
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
  {:else if name === "issue"}
    <path d="M6 13H8V14H6V13Z" />
    <path d="M10 13L8 13V14L10 14V13Z" />
    <path d="M3 5.99999L3 7.99999H2L2 5.99999H3Z" />
    <path d="M13 5.99999V7.99999H14V5.99999H13Z" />
    <path d="M4 12H6V13L4 13V12Z" />
    <path d="M12 12H10V13L12 13V12Z" />
    <path d="M4 3.99999V5.99999H3L3 3.99999H4Z" />
    <path d="M12 3.99999V5.99999L13 5.99999V3.99999H12Z" />
    <path d="M4 9.99999L4 12H3L3 9.99999H4Z" />
    <path d="M12 9.99998V12H13V9.99998H12Z" />
    <path d="M6 3.99998L4 3.99999L4 2.99998L6 2.99999V3.99998Z" />
    <path d="M10 3.99998L12 3.99999V2.99998L10 2.99998V3.99998Z" />
    <path d="M3 7.99999L3 9.99999H2L2 7.99999H3Z" />
    <path d="M13 7.99999V9.99998L14 9.99999V7.99999H13Z" />
    <path d="M8 2.99998L6 2.99999V1.99998L8 1.99998V2.99998Z" />
    <path d="M8 2.99998L10 2.99998L10 1.99998L8 1.99998V2.99998Z" />
    <path d="M7 5.99999H9V6.99999H7V5.99999Z" />
    <path d="M7 8.99999H9V9.99998H7V8.99999Z" />
    <path d="M10 6.99998V8.99998L9 8.99999L9 6.99999L10 6.99998Z" />
    <path d="M7 6.99999L7 8.99999L6 8.99998V6.99998L7 6.99999Z" />
  {:else if name === "lock"}
    <path d="M6 2H10V3H6V2Z" />
    <path d="M10 3L11 3V4H10V3Z" />
    <path d="M6 3H5V4H6L6 3Z" />
    <path d="M11 4L12 4V7H11V4Z" />
    <path d="M5 4H4V7H5V4Z" />
    <path d="M2 7H3V13H2V7Z" />
    <path d="M3 13H13V14H3L3 13Z" />
    <path d="M13 7H14V13H13V7Z" />
    <path d="M3 6H13V7H3L3 6Z" />
    <path d="M7 8H9V11H7V8Z" />
  {:else if name === "markdown"}
    <path d="M2 4.00003H3V12H2V4.00003Z" />
    <path d="M3 7.00003H4V12H3V7.00003Z" />
    <path d="M9 4.00003H8V8.00003H9V4.00003Z" />
    <path d="M8 7.00003H7V9.00003H8V7.00003Z" />
    <path d="M11 11H12V12H11V11Z" />
    <path d="M9 10H13V11H9V10Z" />
    <path d="M8 9.00003H14V10H8V9.00003Z" />
    <path d="M10 4.00003H12V10H10V4.00003Z" />
    <path d="M4 5.00003H5V7.00003H4V5.00003Z" />
    <path d="M5 7.00003H6V8.00003H5V7.00003Z" />
    <path d="M5 6.00003H6V7.00003H5V6.00003Z" />
    <path d="M6 5.00003H7V7.00003H6V5.00003Z" />
    <path d="M7 4.00003H8V6.00003H7V4.00003Z" />
    <path d="M8 11H9V12H8V11Z" />
    <path d="M7 10H8V12H7V10Z" />
    <path d="M3 4.00003H4V6.00003H3V4.00003Z" />
  {:else if name === "moon"}
    <path d="M4 3H6V4H4V3Z" />
    <path d="M3 4L4 4L4 6H3V4Z" />
    <path d="M2 6L3 6L3 10H2V6Z" />
    <path d="M3 10H4V12H3L3 10Z" />
    <path d="M4 12H6V13H4L4 12Z" />
    <path d="M6 13H10V14H6L6 13Z" />
    <path d="M6 13H10V14H6L6 13Z" />
    <path d="M12 10H13V12H12V10Z" />
    <path d="M8 9H10V10H8V9Z" />
    <path d="M6 6H7V7H6V6Z" />
    <path d="M6 7H7V8H6V7Z" />
    <path d="M7 8H8V9H7V8Z" />
    <path d="M10 10H12V11H10V10Z" />
    <path d="M5 5H6V6H5V5Z" />
    <path d="M10 12H12V13H10L10 12Z" />
    <path d="M5 4H6L6 5H5L5 4Z" />
    <path d="M13 9H14V11H13V9Z" />
    <path d="M5 2H7V3H5V2Z" />
    <path d="M5 2H7V3H5V2Z" />
    <path d="M5 2H7V3H5V2Z" />
    <path d="M11 4H12V7H11V4Z" />
    <path d="M10 5H13V6H10V5Z" />
    <path d="M9 3H10V4H9V3Z" />
  {:else if name === "more-vertical"}
    <path d="M9 2V4L7 4L7 2L9 2Z" />
    <path d="M9 7L9 9H7L7 7H9Z" />
    <path d="M9 12L9 14H7L7 12H9Z" />
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
  {:else if name === "online"}
    <path d="M3 5.99999L3 7.99999H2L2 5.99999H3Z" />
    <path d="M13 9.99998V7.99998H14V9.99998H13Z" />
    <path d="M5 8.99998L5 6.99998H4L4 8.99998H5Z" />
    <path d="M11 6.99999V8.99999H12V6.99999H11Z" />
    <path d="M4 12H5V13H4V12Z" />
    <path d="M12 3.99998L11 3.99998V2.99998L12 2.99998V3.99998Z" />
    <path d="M4 3.99999V5.99999L3 5.99999L3 3.99999H4Z" />
    <path d="M12 12V9.99998H13V12H12Z" />
    <path d="M4 9.99998L4 12H3L3 9.99998H4Z" />
    <path d="M12 5.99999V3.99998L13 3.99999V5.99999H12Z" />
    <path d="M5 3.99999H4L4 2.99999L5 2.99999V3.99999Z" />
    <path d="M11 12H12V13H11V12Z" />
    <path d="M3 7.99999L3 9.99998H2L2 7.99999H3Z" />
    <path d="M13 7.99998V5.99999L14 5.99998V7.99998H13Z" />
    <path d="M6 11H5L5 8.99998H6L6 11Z" />
    <path d="M10 4.99998H11L11 6.99999L10 6.99998L10 4.99998Z" />
    <path d="M6 6.99998H5L5 4.99998L6 4.99998L6 6.99998Z" />
    <path d="M10 8.99998L11 8.99999L11 11H10L10 8.99998Z" />
    <path d="M7 6.99998H9V8.99998H7V6.99998Z" />
    <path d="M6 11H7V12H6V11Z" />
    <path d="M10 4.99998L9 4.99998V3.99998L10 3.99998V4.99998Z" />
    <path d="M6 3.99999H7V4.99999L6 4.99998L6 3.99999Z" />
    <path d="M10 12H9V11H10L10 12Z" />
    <path d="M5 1.99999H6V2.99999L5 2.99999L5 1.99999Z" />
    <path d="M11 14H10V13H11V14Z" />
    <path d="M5 13H6V14H5V13Z" />
    <path d="M11 2.99998L10 2.99998V1.99998L11 1.99998V2.99998Z" />
  {:else if name === "patch"}
    <path d="M2 3H3V13H2V3Z" />
    <path d="M3 13H12V14H3L3 13Z" />
    <path d="M3 2H12V3L3 3L3 2Z" />
    <path d="M12 3L13 3V13H12V3Z" />
    <path d="M7 4H8V9H7V4Z" />
    <path d="M5 6H10V7H5V6Z" />
    <path d="M5 10H10V11H5V10Z" />
  {:else if name === "pen"}
    <path d="M13 4.99998H14V5.99999H13V4.99998Z" />
    <path d="M2 13H3V14H2V13Z" />
    <path d="M10 1.99998H11V2.99998H10V1.99998Z" />
    <path d="M11 2.99998L12 2.99998V3.99998H11V2.99998Z" />
    <path d="M12 3.99998L13 3.99998V4.99998H12V3.99998Z" />
    <path d="M6 5.99998H7V6.99998H6V5.99998Z" />
    <path d="M7 4.99998H8V5.99998H7V4.99998Z" />
    <path d="M8 3.99998H9V4.99998H8V3.99998Z" />
    <path d="M9 4.99998L10 4.99998V5.99998H9V4.99998Z" />
    <path d="M10 5.99998H11V6.99998H10V5.99998Z" />
    <path d="M9 2.99998L10 2.99998V3.99998H9V2.99998Z" />
    <path d="M12 5.99998L13 5.99999V6.99998H12V5.99998Z" />
    <path d="M5 6.99999L6 6.99998L6 7.99999H5V6.99999Z" />
    <path d="M9 8.99998H10V9.99998H9V8.99998Z" />
    <path d="M10 7.99998H11V8.99998H10V7.99998Z" />
    <path d="M11 6.99998H12V7.99998H11V6.99998Z" />
    <path d="M4 7.99998L5 7.99999V8.99998H4V7.99998Z" />
    <path d="M8 9.99998H9L9 11H8V9.99998Z" />
    <path d="M3 8.99998H4V9.99998H3V8.99998Z" />
    <path d="M7 11H8V12H7V11Z" />
    <path d="M2 9.99998H3V11H2V9.99998Z" />
    <path d="M6 12H7V13H6V12Z" />
    <path d="M4 12H5V13H4V12Z" />
    <path d="M4 11H5V12H4V11Z" />
    <path d="M3 11H4V12H3V11Z" />
    <path d="M3 9.99998H4V11H3V9.99998Z" />
    <path d="M5 12H6V13H5V12Z" />
    <path d="M4 13H5V14H4V13Z" />
    <path d="M5 13H6L6 14H5V13Z" />
    <path d="M2 12H3V13H2V12Z" />
    <path d="M2 11H3V12H2V11Z" />
    <path d="M3 12H4V13H3V12Z" />
    <path d="M3 13H4V14H3V13Z" />
  {:else if name === "plus"}
    <path d="M7.00002 2H9.00002V14H7.00002V2Z" />
    <path d="M14 7V9L2.00002 9L2.00002 7L14 7Z" />
  {:else if name === "reply"}
    <path d="M2.5 9V8H3.5V9H2.5Z" />
    <path d="M3.5 10L3.5 9L4.5 9V10L3.5 10Z" />
    <path d="M3.5 7V8L4.5 8V7L3.5 7Z" />
    <path d="M3.5 8L13.5 8V9L3.5 9V8Z" />
    <path d="M4.5 6H5.5V11H4.5V6Z" />
    <path d="M13.5 4H12.5V8L13.5 8L13.5 4Z" />
    <path d="M5.5 5H6.5V12H5.5V5Z" />
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
  {:else if name === "revision"}
    <path d="M6 13H8V14H6V13Z" />
    <path d="M8 13H9V14H8V13Z" />
    <path d="M3 6L3 8H2L2 6H3Z" />
    <path d="M4 12H6L6 13H4V12Z" />
    <path d="M12 12H10V13H12V12Z" />
    <path d="M4 4V6H3L3 4H4Z" />
    <path d="M4 10L4 12H3L3 10H4Z" />
    <path d="M12 10V11H13V10H12Z" />
    <path d="M13 7V8H14V7H13Z" />
    <path d="M6 4L4 4L4 3L6 3V4Z" />
    <path d="M3 8L3 10H2L2 8H3Z" />
    <path d="M8 3L6 3L6 2L8 2V3Z" />
    <path d="M8 3H9V2L8 2V3Z" />
    <path d="M11 4H12V5H11V4Z" />
    <path d="M12 3H13V4L12 4V3Z" />
    <path d="M11 6H12V7H11V6Z" />
    <path d="M12 5L13 5V6H12V5Z" />
    <path d="M13 4H14V5H13V4Z" />
    <path d="M10 7L11 7V8H10V7Z" />
    <path d="M7 6H8V7H7V6Z" />
    <path d="M8 5H9V6H8V5Z" />
    <path d="M9 4H10V5L9 5V4Z" />
    <path d="M10 3H11V4L10 4L10 3Z" />
    <path d="M11 2H12V3L11 3V2Z" />
    <path d="M9 8L10 8L10 9H9V8Z" />
    <path d="M7 9H8V10H7V9Z" />
    <path d="M7 8H8V9H7V8Z" />
    <path d="M6 8H7V9H6V8Z" />
    <path d="M6 7H7V8H6V7Z" />
    <path d="M8 9H9V10L8 10V9Z" />
    <path d="M6 9H7V10H6V9Z" />
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
  {:else if name === "seedling-filled"}
    <path d="M10 6H9V5L10 5V6Z" />
    <path d="M11 5L10 5V4L11 4V5Z" />
    <path d="M12 4H11L11 3L12 3L12 4Z" />
    <path d="M13 4L12 4L12 3L13 3L13 4Z" />
    <path d="M14 5L13 5V4L14 4V5Z" />
    <path d="M13 6H12V5H13V6Z" />
    <path d="M5 7L5 6L4 6L4 7L5 7Z" />
    <path d="M4 8V7L3 7L3 8L4 8Z" />
    <path d="M3 9L3 8L2 8L2 9H3Z" />
    <path d="M3 10L3 9H2L2 10H3Z" />
    <path d="M4 11L4 10H3L3 11L4 11Z" />
    <path d="M10 7H9L9 6H10L10 7Z" />
    <path d="M9 8H8V7L9 7L9 8Z" />
    <path d="M8 8H7V7H8V8Z" />
    <path d="M11 8L10 8L10 7L11 7V8Z" />
    <path d="M12 8L10 8L10 7L12 7V8Z" />
    <path d="M13 7H12V6H13V7Z" />
    <path d="M5 10L5 9H4L4 10L5 10Z" />
    <path d="M9 8H8V11H9V8Z" />
    <path d="M8 11L7 11V14H8V11Z" />
    <path d="M6 7L6 6H5L5 7H6Z" />
    <path d="M7 8V7L6 7L6 8H7Z" />
    <path d="M7 9L7 7L6 7L6 9H7Z" />
    <path d="M6 10V9H5L5 10H6Z" />
    <path d="M4 7L6 7L6 9H4L4 7Z" />
    <path d="M3 8L4 8L4 10H3L3 8Z" />
    <path d="M11 4L13 4V5L11 5V4Z" />
    <path d="M10 5L12 5V7L10 7L10 5Z" />
  {:else if name === "settings"}
    <path d="M7 5H14V6H7V5Z" />
    <path d="M9 11L2 11L2 10L9 10V11Z" />
    <path d="M2 5H5V6H2V5Z" />
    <path d="M14 11H11V10H14V11Z" />
    <path d="M9 8H11V9H9V8Z" />
    <path d="M5 3H7V4H5V3Z" />
    <path d="M9 12H11V13H9V12Z" />
    <path d="M5 7H7V8H5V7Z" />
    <path d="M8 9L9 9V12H8V9Z" />
    <path d="M4 4L5 4L5 7H4V4Z" />
    <path d="M11 9L12 9V12H11V9Z" />
    <path d="M7 4L8 4V7L7 7V4Z" />
  {:else if name === "sidebar"}
    <path d="M2 3H3V13H2V3Z" />
    <path d="M13 3H14V13H13V3Z" />
    <path d="M6 2H7V14H6V2Z" />
    <path d="M3 2H13L13 3H3L3 2Z" />
    <path d="M3 13H13L13 14H3L3 13Z" />
  {:else if name === "sun"}
    <path d="M8 2H9V3H8V2Z" />
    <path d="M14 8V9H13V8H14Z" />
    <path d="M4 7V8H3V7H4Z" />
    <path d="M7 12H8V13H7V12Z" />
    <path d="M8 3H9L9 4H8L8 3Z" />
    <path d="M13 8V9H12V8H13Z" />
    <path d="M3 7V8H2L2 7H3Z" />
    <path d="M7 13H8V14H7V13Z" />
    <path d="M7 5H9V6H7V5Z" />
    <path d="M7 10H9V11H7V10Z" />
    <path d="M5 7H6V9H5V7Z" />
    <path d="M10 7H11V9H10V7Z" />
    <path d="M11 5H12V6H11V5Z" />
    <path d="M10 12H11V11H10V12Z" />
    <path d="M6 4H5V5H6V4Z" />
    <path d="M5 11H4V10H5V11Z" />
    <path d="M12 4H13V5H12V4Z" />
    <path d="M11 13H12V12H11V13Z" />
    <path d="M5 3H4V4H5L5 3Z" />
    <path d="M4 12H3V11H4L4 12Z" />
    <path d="M6 6H7V7L6 7V6Z" />
    <path d="M6 9L7 9L7 10H6V9Z" />
    <path d="M9 9L10 9L10 10H9L9 9Z" />
    <path d="M9 6H10V7H9V6Z" />
  {:else if name === "warning"}
    <path d="M7 2H9V3H7V2Z" />
    <path d="M6 3H7V5H6V3Z" />
    <path d="M5 5H6V7H5V5Z" />
    <path d="M4 7H5V9H4V7Z" />
    <path d="M3 9H4V11H3V9Z" />
    <path d="M2 11H3V13H2V11Z" />
    <path d="M10 3H9V5H10V3Z" />
    <path d="M11 5H10V7H11V5Z" />
    <path d="M12 7H11V9H12V7Z" />
    <path d="M13 9H12V11H13V9Z" />
    <path d="M14 11H13V13H14V11Z" />
    <path d="M3 13H13V14H3V13Z" />
    <path d="M7 6H9V10H7V6Z" />
    <path d="M7 11H9V12H7V11Z" />
  {:else}
    {unreachable(name)}
  {/if}
</svg>
