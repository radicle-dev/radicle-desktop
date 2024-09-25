<script lang="ts">
  import dompurify from "dompurify";
  import escape from "lodash/escape";

  export let content: string;
  export let fontSize: "tiny" | "small" | "regular" | "medium" | "large" =
    "small";

  function formatInlineTitle(input: string): string {
    return input.replaceAll(/`([^`]+)`/g, "<code>$1</code>");
  }
</script>

<style>
  .content :global(code) {
    font-family: var(--font-family-monospace);
    padding: 0.125rem 0.25rem;
    background-color: var(--color-fill-ghost);
    font-size: inherit;
  }
</style>

<span
  class="content"
  class:txt-large={fontSize === "large"}
  class:txt-medium={fontSize === "medium"}
  class:txt-regular={fontSize === "regular"}
  class:txt-small={fontSize === "small"}
  class:txt-tiny={fontSize === "tiny"}>
  {@html dompurify.sanitize(formatInlineTitle(escape(content)))}
</span>
