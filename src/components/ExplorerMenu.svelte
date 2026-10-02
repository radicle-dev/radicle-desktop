<script module lang="ts">
  export interface MenuPosition {
    x: number;
    y: number;
    target: HTMLElement;
  }

  // WebKit selects the word under a right click, which would highlight the
  // label behind the menu.
  export function keepSelection(event: MouseEvent) {
    if (event.button === 2) event.preventDefault();
  }

  export function menuPosition(event: MouseEvent): MenuPosition {
    event.preventDefault();
    event.stopPropagation();
    window.getSelection()?.removeAllRanges();

    return {
      x: event.clientX,
      y: event.clientY,
      target: event.currentTarget as HTMLElement,
    };
  }
</script>

<script lang="ts">
  import type { Config } from "@bindings/config/Config";

  import { writeToClipboard } from "@app/lib/invoke";
  import { explorerHost } from "@app/lib/utils";

  import ContextMenu from "@app/components/ContextMenu.svelte";
  import Icon from "@app/components/Icon.svelte";

  interface Props {
    x: number;
    y: number;
    target: HTMLElement;
    url: string;
    /** The canonical `rad:` URI of what is linked. */
    uri: string;
    config: Config;
    onclose: () => void;
  }

  const { x, y, target, url, uri, config, onclose }: Props = $props();
</script>

<ContextMenu {x} {y} {target} {onclose}>
  <a
    class="menu-item"
    role="menuitem"
    href={url}
    target="_blank"
    rel="noreferrer noopener">
    <Icon name="open-external" />
    Open in {explorerHost(config)}
  </a>
  <button
    class="menu-item"
    role="menuitem"
    onclick={() => writeToClipboard(url)}>
    <Icon name="link" />
    Copy link to {explorerHost(config)}
  </button>
  <div class="menu-separator"></div>
  <button
    class="menu-item"
    role="menuitem"
    onclick={() => writeToClipboard(uri)}>
    <Icon name="copy" />
    Copy rad: URI
  </button>
</ContextMenu>
