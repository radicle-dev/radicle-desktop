<script lang="ts">
  import type { Config } from "@bindings/config/Config";

  import { didFromPublicKey, truncateDid } from "@app/lib/utils";
  import { radicleInstalled } from "@app/lib/checkRadicleCLI.svelte";

  import Border from "@app/components/Border.svelte";
  import Command from "@app/components/Command.svelte";
  import CopyableId from "@app/components/CopyableId.svelte";
  import Icon from "@app/components/Icon.svelte";
  import NakedButton from "@app/components/NakedButton.svelte";
  import NodeId from "@app/components/NodeId.svelte";
  import Popover from "@app/components/Popover.svelte";
  import RepoGuide from "@app/components/RepoGuide.svelte";

  interface Props {
    config: Config;
  }
  const { config }: Props = $props();
</script>

<style>
  .guide-header {
    padding-bottom: 1rem;
  }
  .spacer {
    width: 100%;
    border-bottom: 1px solid var(--color-border-default);
    height: 1px;
    margin: 1rem 0;
  }
</style>

<Popover
  popoverId="popover-guide"
  popoverPadding="0"
  popoverPositionTop="2.5rem"
  popoverPositionRight="-9.3rem">
  {#snippet toggle(onclick)}
    <NakedButton variant="ghost" {onclick} stylePadding="0 4px">
      <Icon name="info" />
    </NakedButton>
  {/snippet}
  {#snippet popover()}
    <Border
      variant="ghost"
      styleGap="0"
      stylePadding="1rem"
      styleMinWidth="36rem"
      styleOverflow="auto"
      styleMaxHeight="calc(100vh - 5rem)"
      styleAlignItems="flex-start"
      styleFlexDirection="column">
      <div
        style:position="relative"
        style:display="flex"
        style:gap="0.5rem"
        style:flex-direction="column"
        style:padding="1rem"
        style:margin-bottom="1rem"
        style:width="100%"
        style:background-color="var(--color-background-float)">
        <div class="txt-semibold txt-medium" style:margin-bottom="1rem">
          Getting started
        </div>
        <div class="txt-small" style:display="inline">
          Hello <span style:padding-left="0.25rem">
            <NodeId inline publicKey={config.publicKey} alias={config.alias} />,
          </span>
          your identity has been created and stored on your machine.
        </div>
        <div class="txt-small">
          Your public key is <CopyableId
            inline
            id={didFromPublicKey(config.publicKey)}>
            {truncateDid(config.publicKey)}
          </CopyableId>
          you can share this with anyone to find you on the network.
        </div>
        <div class="spacer"></div>
        {#if radicleInstalled()}
          <div class="global-flex txt-small">
            <Icon name="checkbox-checked" />Radicle CLI is setup
          </div>
        {:else}
          <div class="txt-small">
            <div class="global-flex" style:padding-bottom="0.5rem">
              <Icon name="checkbox-unchecked" />Make sure to install Radicle CLI
            </div>
            <div style:padding-bottom="0.5rem">
              To be able to interact with repos on the Radicle network you'll
              need to install a node on your computer. This node will identify
              itself on the network with your keys to push and pull changes.
            </div>
            <div style:padding-bottom="0.5rem">
              To install the node and other Radicle CLI tooling, simply run the
              command below from your shell:
            </div>
            <Command
              styleWidth="fit-content"
              command="curl -sSf https://radicle.xyz/install | sh" />
          </div>
        {/if}
      </div>
      <div class="guide-header txt-medium txt-semibold">Guide</div>

      <RepoGuide />
    </Border>
  {/snippet}
</Popover>
