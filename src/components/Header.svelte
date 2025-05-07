<script lang="ts">
  import type { Snippet } from "svelte";
  import type { Config } from "@bindings/config/Config";

  import { boolean } from "zod";
  import { onMount } from "svelte";

  import * as router from "@app/lib/router";
  import useLocalStorage from "@app/lib/useLocalStorage.svelte";
  import {
    checkRadicleCLI,
    radicleInstalled,
  } from "@app/lib/checkRadicleCLI.svelte";
  import { dynamicInterval } from "@app/lib/interval";
  import { nodeRunning } from "@app/lib/events";
  import { didFromPublicKey, truncateDid } from "@app/lib/utils";

  import Avatar from "@app/components/Avatar.svelte";
  import Border from "@app/components/Border.svelte";
  import Command from "@app/components/Command.svelte";
  import CopyableId from "@app/components/CopyableId.svelte";
  import Repos from "@app/views/home/guides/Repos.svelte";
  import Icon from "@app/components/Icon.svelte";
  import NakedButton from "@app/components/NakedButton.svelte";
  import NodeId from "@app/components/NodeId.svelte";
  import Popover, { setFocused } from "@app/components/Popover.svelte";

  const activeRouteStore = router.activeRouteStore;

  const firstLaunchStorage = useLocalStorage(
    "appFirstLaunch",
    boolean(),
    true,
    !window.localStorage,
  );

  interface Props {
    config: Config;
    center?: Snippet;
  }

  onMount(async () => {
    try {
      await checkRadicleCLI();
      dynamicInterval("checkRadicleCLI", checkRadicleCLI, 30_000);
    } catch {
      dynamicInterval("checkRadicleCLI", checkRadicleCLI, 1_000);
    }

    if (firstLaunchStorage.value === true) {
      setFocused("popover-guide");
      firstLaunchStorage.value = false;
    }
  });

  const { center, config }: Props = $props();
</script>

<style>
  .header {
    height: 3rem;
    padding: 0.5rem 1rem;
    display: flex;
    align-items: flex-start;
  }
  .header:after {
    content: " ";
    position: absolute;
    top: 0;
    left: 0.5rem;
    right: 0.5rem;
    height: 3rem;
    z-index: -1;
    background-color: var(--color-background-float);
    clip-path: var(--3px-bottom-corner-fill);
  }
  .wrapper {
    display: flex;
    flex-direction: column;
    width: 100%;
    row-gap: 8px;
  }
  .top-row {
    display: flex;
    width: 100%;
    justify-content: space-between;
  }
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

<div class="header global-flex">
  <div class="wrapper">
    <div class="top-row">
      <div class="global-flex" style:gap="0.25rem">
        <NakedButton
          variant="ghost"
          onclick={() => {
            void router.push({ resource: "home" });
          }}
          stylePadding="0 4px">
          <Avatar publicKey={config.publicKey} />
        </NakedButton>
        <NakedButton
          variant="ghost"
          onclick={() => {
            window.history.back();
          }}
          stylePadding="0 4px">
          <Icon name="arrow-left" />
        </NakedButton>
        <NakedButton
          variant="ghost"
          onclick={() => {
            window.history.forward();
          }}
          stylePadding="0 4px">
          <Icon name="arrow-right" />
        </NakedButton>
      </div>

      {@render center?.()}

      <div class="global-flex">
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
                    <NodeId
                      inline
                      publicKey={config.publicKey}
                      alias={config.alias} />,
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
                      <Icon name="checkbox-unchecked" />Make sure to install
                      Radicle CLI
                    </div>
                    <div style:padding-bottom="0.5rem">
                      To be able to interact with repos on the Radicle network
                      you'll need to install a node on your computer. This node
                      will identify itself on the network with your keys to push
                      and pull changes.
                    </div>
                    <div style:padding-bottom="0.5rem">
                      To install the node and other Radicle CLI tooling, simply
                      run the command below from your shell:
                    </div>
                    <Command
                      styleWidth="fit-content"
                      command="curl -sSf https://radicle.xyz/install | sh" />
                  </div>
                {/if}
              </div>
              <div class="guide-header txt-medium txt-semibold">Guide</div>

              <Repos />
            </Border>
          {/snippet}
        </Popover>
        <Popover
          popoverPadding="0"
          popoverPositionTop="2.5rem"
          popoverPositionRight="0">
          {#snippet toggle(onclick)}
            <NakedButton variant="ghost" {onclick}>
              {#if $nodeRunning}
                <Icon name="online" />
                Online
              {:else}
                <Icon name="offline" />
                Offline
              {/if}
            </NakedButton>
          {/snippet}
          {#snippet popover()}
            <Border
              variant="ghost"
              stylePadding="1rem"
              styleMinWidth="20rem"
              styleAlignItems="flex-start"
              styleFlexDirection="column">
              <div class="txt-small txt-missing">
                {#if $nodeRunning}
                  Your node is up and running, your changes will be synced
                  automatically.
                {:else}
                  Your node is not running, changes you make are safe but won't
                  be announced.
                {/if}
              </div>
            </Border>
          {/snippet}
        </Popover>
        <NakedButton
          variant="ghost"
          stylePadding="0 4px"
          active={$activeRouteStore.resource === "inbox"}
          onclick={() => router.push({ resource: "inbox" })}>
          <Icon name="inbox" />
        </NakedButton>
      </div>
    </div>
  </div>
</div>
