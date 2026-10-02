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
