<script lang="ts">
  import type { ComponentProps } from "svelte";

  import debounce from "lodash/debounce";
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";

  import Icon from "./Icon.svelte";

  const {
    id,
  }: {
    id: string;
  } = $props();

  let icon: ComponentProps<typeof Icon>["name"] = $state("copy");

  const restoreIcon = debounce(() => {
    icon = "copy";
  }, 1000);

  async function copy() {
    await writeText(id);
    icon = "checkmark";
    restoreIcon();
  }
</script>

<style>
  .copyable-id {
    cursor: pointer;
    color: var(--color-foreground-dim);
  }

  .copyable-id:hover {
    color: var(--color-foreground-contrast);
  }
</style>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  role="button"
  tabindex="0"
  onclick={copy}
  class="copyable-id global-flex txt-small txt-monospace">
  {id}
  <Icon name={icon} />
</div>
