<script lang="ts">
  import { writeToClipboard } from "@app/lib/invoke";

  import Icon from "@app/components/Icon.svelte";

  interface Props {
    // Produces the text lazily, so large content isn't built until clicked.
    text: () => string;
    title?: string;
  }

  const { text, title = "Copy" }: Props = $props();

  let copied = $state(false);
  let timer: ReturnType<typeof setTimeout> | undefined;

  async function copy() {
    await writeToClipboard(text());
    copied = true;
    clearTimeout(timer);
    timer = setTimeout(() => {
      copied = false;
    }, 1000);
  }

  $effect(() => () => clearTimeout(timer));
</script>

<style>
  .copy {
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    color: var(--color-text-secondary);
    display: flex;
    align-items: center;
  }
  .copy:hover {
    color: var(--color-text-primary);
  }
</style>

<button class="copy" {title} onclick={copy}>
  <Icon name={copied ? "checkmark" : "clipboard"} />
</button>
