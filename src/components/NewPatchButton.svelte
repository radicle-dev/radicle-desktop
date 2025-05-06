<script lang="ts">
  import Border from "./Border.svelte";
  import Button from "./Button.svelte";
  import Command from "./Command.svelte";
  import Icon from "./Icon.svelte";
  import OutlineButton from "./OutlineButton.svelte";
  import Popover from "./Popover.svelte";

  interface Props {
    outline?: boolean;
    rid: string;
  }

  const { outline = false, rid }: Props = $props();
</script>

<Popover popoverPositionRight="0" popoverPositionTop="3rem">
  {#snippet toggle(onclick)}
    {#if outline}
      <OutlineButton styleHeight="2.5rem" variant="ghost" {onclick}>
        <Icon name="add" />New patch<Icon name="chevron-down" />
      </OutlineButton>
    {:else}
      <Button styleHeight="2.5rem" variant="secondary" {onclick}>
        <Icon name="add" />New patch<Icon name="chevron-down" />
      </Button>
    {/if}
  {/snippet}

  {#snippet popover()}
    <div class="txt-small">
      <Border
        styleAlignItems="flex-start"
        styleBackgroundColor="var(--color-background-float)"
        styleFlexDirection="column"
        styleGap="2rem"
        stylePadding="1rem"
        styleWidth="28rem"
        variant="ghost">
        <div>
          <div class="txt-semibold" style:margin-bottom="0.5rem">
            Create a new patch
          </div>
          <div
            class="global-flex"
            style:flex-direction="column"
            style:align-items="flex-start"
            style:gap="0.5rem">
            Create a new branch in your working copy, commit your changes, and
            run:
            <Command
              command="git push rad HEAD:refs/patches"
              styleWidth="100%" />
          </div>
        </div>

        <div style:margin-bottom="1rem">
          <div class="txt-semibold" style:margin-bottom="0.5rem">
            Don't have a working copy yet?
          </div>
          <div
            class="global-flex"
            style:flex-direction="column"
            style:align-items="flex-start"
            style:gap="0.5rem">
            To checkout a working copy of this repo, run:
            <Command command={`rad checkout ${rid}`} styleWidth="100%" />
          </div>
        </div>
      </Border>
    </div>
  {/snippet}
</Popover>
