<script lang="ts">
  import dompurify from "dompurify";
  import escape from "lodash/escape";

  interface Props {
    content: string;
    fontSize?: "tiny" | "small" | "regular" | "medium" | "large";
  }

  const { content, fontSize = "small" }: Props = $props();

  function formatInlineTitle(input: string): string {
    return input
      .replaceAll(/`([^`]+)`/g, "<code>$1</code>")
      .replaceAll(/~~([^~~]+)~~/g, "<del>$1</del>");
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
