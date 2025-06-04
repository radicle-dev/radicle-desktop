<script lang="ts" module>
  export const guidePopoverToggleId = "guide-popover-toggle";
</script>

<script lang="ts">
  import type { Config } from "@bindings/config/Config";

  import { activeRouteStore, push } from "@app/lib/router";
  import { addRepoPopoverToggleId } from "./AddRepoButton.svelte";
  import { didFromPublicKey, truncateDid } from "@app/lib/utils";
  import { nodeRunning } from "@app/lib/events";
  import { radicleInstalled } from "@app/lib/checkRadicleCLI.svelte";
  import { sleep } from "@app/lib/sleep";

  import Border from "@app/components/Border.svelte";
  import Command from "@app/components/Command.svelte";
  import CopyableId from "@app/components/CopyableId.svelte";
  import Icon from "@app/components/Icon.svelte";
  import NakedButton from "@app/components/NakedButton.svelte";
  import NodeId from "@app/components/NodeId.svelte";
  import Popover from "@app/components/Popover.svelte";

  interface Props {
    config: Config;
  }
  const { config }: Props = $props();

  let popoverExpanded: boolean = $state(false);
</script>

<style>
  .spacer {
    width: 100%;
    border-bottom: 1px solid var(--color-border-default);
    height: 1px;
    margin: 1rem 0;
  }

  button {
    text-decoration: underline;
    border: 0;
    color: var(--color-foreground-contrast);
    margin: 0;
    padding: 0;
    background-color: transparent;
    cursor: pointer;
  }
</style>

<Popover
  popoverPadding="0"
  popoverPositionTop="2.5rem"
  bind:expanded={popoverExpanded}
  popoverPositionRight="-9.3rem">
  {#snippet toggle(onclick)}
    <NakedButton
      id={guidePopoverToggleId}
      variant="ghost"
      {onclick}
      stylePadding="0 0.25rem"
      active={popoverExpanded}>
      <Icon name="info" /> Guide
    </NakedButton>
  {/snippet}
  {#snippet popover()}
    <Border
      variant="ghost"
      styleGap="0"
      stylePadding="1rem"
      styleMinWidth="32rem"
      styleBackgroundColor="var(--color-background-float)"
      styleOverflow="auto"
      styleMaxHeight="calc(100vh - 5rem)"
      styleAlignItems="flex-start"
      styleFlexDirection="column">
      <div
        style:position="relative"
        style:display="flex"
        style:line-height="1.625rem"
        style:gap="0.5rem"
        style:flex-direction="column"
        style:width="100%">
        <div class="txt-semibold txt-medium" style:margin-bottom="1rem">
          Get started
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
        {#if radicleInstalled() || $nodeRunning}
          <div class="global-flex txt-small">
            <div class="global-flex">
              <Icon name="thumb-up" />Radicle CLI is installed, you're good to
              go.
            </div>
            <button
              class="txt-small"
              onclick={async () => {
                if ($activeRouteStore.resource !== "home") {
                  await push({
                    resource: "home",
                    activeTab: "all",
                  });
                }
                await sleep(1);
                const addRepoButton = document.getElementById(
                  addRepoPopoverToggleId,
                );
                addRepoButton?.click();
              }}>
              Try adding a repo!
            </button>
          </div>
        {:else}
          <div class="txt-small">
            <div class="global-flex" style:padding-bottom="1rem">
              <Icon name="warning" />Radicle CLI is not installed
            </div>
            <div style:padding-bottom="1rem">
              To interact with repositories on the Radicle network, you’ll need
              to install Radicle node along with its accompanying CLI tools. The
              node runs in the background, enabling seamless pushing and pulling
              of changes, while the CLI tools let you manage the node and
              provide interoperability between Git and Radicle.
            </div>
            <div style:padding-bottom="0.5rem">
              To install Radicle node and CLI tooling, run this in your shell:
            </div>
            <Command
              styleWidth="fit-content"
              command="curl -sSf https://radicle.xyz/install | sh" />
          </div>
        {/if}
      </div>
    </Border>
  {/snippet}
</Popover>
