<script lang="ts">
  import type { ComponentProps } from "svelte";

  import { writeToClipboard } from "@app/lib/invoke";

  import debounce from "lodash/debounce";
  import Icon from "@app/components/Icon.svelte";
  import Popover, { closeFocused } from "@app/components/Popover.svelte";
  import DropdownListItem from "@app/components/DropdownListItem.svelte";
  import Border from "@app/components/Border.svelte";

  interface Props {
    icon: ComponentProps<typeof Icon>["name"];
    id: string;
    url: string;
  }

  const { icon, id, url }: Props = $props();

  let popoverExpanded: boolean = $state(false);
  let triggerIcon: ComponentProps<typeof Icon>["name"] = $state("copy");
  const restoreIcon = debounce(() => {
    triggerIcon = "copy";
  }, 1000);
</script>

<Popover
  popoverPositionLeft="0"
  popoverPositionTop="2rem"
  bind:expanded={popoverExpanded}>
  {#snippet toggle(onclick)}
    <Icon name={triggerIcon} {onclick} />
  {/snippet}

  {#snippet popover()}
    <Border variant="ghost">
      <div
        class="global-flex txt-monospace"
        style:flex-direction="column"
        style:align-items="flex-start">
        <DropdownListItem
          styleGap="0.5rem"
          selected={false}
          onclick={async () => {
            await writeToClipboard(id);
            triggerIcon = "checkmark";
            restoreIcon();
            closeFocused();
          }}
          styleWidth="100%">
          <div class="global-flex">
            <Icon name={icon} />
            {id}
            <Icon name="copy" />
          </div>
        </DropdownListItem>
        <a
          style:text-decoration="none"
          style:width="100%"
          onclick={closeFocused}
          href={url}
          target="_blank">
          <DropdownListItem styleGap="0.5rem" selected={false}>
            <div class="global-flex" style:width="100%">
              <Icon name="seedling" />
              view on seed.radicle.garden
              <div style:margin-left="auto">
                <Icon name="open-external" />
              </div>
            </div>
          </DropdownListItem>
        </a>
      </div>
    </Border>
  {/snippet}
</Popover>
