<script lang="ts">
  import debounce from "lodash/debounce";

  import Icon from "@app/components/Icon.svelte";
  import { writeToClipboard } from "@app/lib/invoke";

  interface Props {
    text: string;
  }

  const { text }: Props = $props();

  let icon: "copy" | "checkmark" = $state("copy");

  const restoreIcon = debounce(() => {
    icon = "copy";
  }, 800);

  export async function copy() {
    await writeToClipboard(text);
    icon = "checkmark";
    restoreIcon();
  }
</script>

<style>
  .clipboard {
    width: 1.5rem;
    height: 1.5rem;
    cursor: pointer;
    display: inline-flex;
    justify-content: center;
    align-items: center;
    user-select: none;
  }
</style>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<span role="button" tabindex="0" class="clipboard" onclick={copy}>
  <Icon name={icon} />
</span>
