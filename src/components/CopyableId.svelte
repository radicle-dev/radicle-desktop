<script lang="ts">
  import type { ComponentProps } from "svelte";

  import debounce from "lodash/debounce";
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";

  import Icon from "./Icon.svelte";

  export let id: string;

  let icon: ComponentProps<Icon>["name"] = "copy";

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
  }
</style>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  role="button"
  tabindex="0"
  onclick={copy}
  class="copyable-id global-flex txt-small txt-monospace"
  style:color="var(--color-foreground-dim)">
  {id}
  <Icon name={icon} />
</div>
