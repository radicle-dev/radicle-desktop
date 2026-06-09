<script lang="ts">
  import debounce from "lodash/debounce";

  import { writeToClipboard } from "@app/lib/invoke";
  import { formatOid } from "@app/lib/utils";

  interface Props {
    ariaLabel?: string;
    clipboard: string;
    id: string;
    // What the hash is (e.g. "Patch ID"), shown in the tooltip.
    label?: string;
    shorten?: boolean;
  }

  const { ariaLabel, clipboard, id, label, shorten = true }: Props = $props();

  let copied = $state(false);
  const tooltip = $derived(
    copied ? "Copied to clipboard" : label ? `Copy ${label}` : "Click to copy",
  );
  const resetCopied = debounce(() => {
    copied = false;
  }, 1000);

  async function copy() {
    await writeToClipboard(clipboard);
    copied = true;
    resetCopied();
  }
</script>

<style>
  .txt-id {
    display: inline-flex;
    align-items: center;
    padding: 0 0.25rem;
    border-radius: var(--border-radius-sm);
    cursor: pointer;
  }
  .txt-id:hover,
  .txt-id:focus-visible {
    background-color: var(--color-surface-subtle);
    color: var(--color-text-primary);
  }
</style>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  class="txt-id"
  title={tooltip}
  aria-label={ariaLabel}
  role="button"
  tabindex="0"
  onclick={async event => {
    event.stopPropagation();
    await copy();
  }}>
  {#if shorten}
    {formatOid(id)}
  {:else}
    {id}
  {/if}
</div>
