<script lang="ts">
  import type { ComponentProps } from "svelte";

  import debounce from "lodash/debounce";

  import { writeToClipboard } from "@app/lib/invoke";
  import { formatOid } from "@app/lib/utils";

  import Icon from "@app/components/Icon.svelte";

  let icon: ComponentProps<typeof Icon>["name"] = $state("copy");
  const text = "Click to copy";
  let tooltip = $state(text);

  const restoreIcon = debounce(() => {
    icon = "copy";
    tooltip = text;
  }, 1000);

  async function copy() {
    await writeToClipboard(clipboard);
    icon = "checkmark";
    tooltip = "Copied to clipboard";
    restoreIcon();
  }

  let visible: boolean = $state(false);

  interface Props {
    ariaLabel?: string;
    clipboard: string;
    id: string;
    shorten?: boolean;
    variant: "oid" | "commit" | "none";
  }

  const { ariaLabel, clipboard, id, shorten = true, variant }: Props = $props();

  const setVisible = debounce((value: boolean) => {
    visible = value;
  }, 50);
</script>

<style>
  .container {
    position: relative;
    display: inline-block;
  }
  .popover {
    position: absolute;
    display: flex;
    align-items: center;
    flex-direction: row;
    gap: 0.5rem;
    justify-content: center;
    z-index: 20;
    background: var(--color-fill-counter);
    color: var(--color-foreground-contrast);
    box-shadow: var(--elevation-low);
    font-family: var(--font-family-sans-serif);
    font-size: var(--font-size-small);
    font-weight: var(--font-weight-regular);
    white-space: nowrap;
    padding: 0.125rem 0.5rem;
    clip-path: var(--1px-corner-fill);
  }
  .target-commit:hover {
    color: var(--color-foreground-contrast);
  }
  .target-oid:hover {
    color: var(--color-foreground-emphasized-hover);
  }
</style>

<div class="container">
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div
    onmouseenter={() => {
      setVisible(true);
    }}
    onmouseleave={() => {
      setVisible(false);
    }}
    class="target-{variant} global-{variant}"
    style:cursor="pointer"
    aria-label={ariaLabel}
    onclick={async event => {
      event.stopPropagation();
      await copy();
      setVisible(true);
    }}
    role="button"
    tabindex="0">
    {#if shorten}
      {formatOid(id)}
    {:else}
      {id}
    {/if}
  </div>

  {#if visible}
    <div style:position="absolute">
      <div class="popover" style:bottom="1.5rem" style:left="1rem">
        <Icon name={icon} />
        {tooltip}
      </div>
    </div>
  {/if}
</div>
