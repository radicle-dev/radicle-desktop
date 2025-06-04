<script lang="ts">
  import type { Config } from "@bindings/config/Config";
  import type { Snippet } from "svelte";

  import { boolean } from "zod";
  import { onMount } from "svelte";

  import useLocalStorage from "@app/lib/useLocalStorage.svelte";

  import { checkRadicleCLI } from "@app/lib/checkRadicleCLI.svelte";
  import { dynamicInterval } from "@app/lib/interval";
  import { sleep } from "@app/lib/sleep";

  import Icon from "@app/components/Icon.svelte";
  import InboxButton from "@app/components/InboxButton.svelte";
  import GuideButton, {
    guidePopoverToggleId,
  } from "@app/components/GuideButton.svelte";
  import NakedButton from "@app/components/NakedButton.svelte";
  import NodeStatusButton from "@app/components/NodeStatusButton.svelte";

  const firstLaunchStorage = useLocalStorage(
    "appFirstLaunch",
    boolean(),
    true,
    !window.localStorage,
  );

  interface Props {
    breadcrumbs?: Snippet;
    config: Config;
    notificationCount: number;
  }

  const { breadcrumbs, config, notificationCount }: Props = $props();

  onMount(async () => {
    try {
      await checkRadicleCLI();
      dynamicInterval("checkRadicleCLI", checkRadicleCLI, 30_000);
    } catch {
      dynamicInterval("checkRadicleCLI", checkRadicleCLI, 1_000);
    }

    if (firstLaunchStorage.value === true) {
      const guidePopoverButton = document.getElementById(guidePopoverToggleId);
      await sleep(1);
      guidePopoverButton?.click();
      firstLaunchStorage.value = false;
    }
  });
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
    z-index: 50;
  }
  .top-row {
    display: flex;
    width: 100%;
    justify-content: space-between;
  }
</style>

<div class="header global-flex">
  <div class="wrapper">
    <div class="top-row">
      <div class="global-flex" style:gap="0.25rem">
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
        <div
          class="global-flex txt-small txt-semibold"
          style:gap="0.25rem"
          style:margin-left="0.5rem">
          {@render breadcrumbs?.()}
        </div>
      </div>

      <div class="global-flex">
        <GuideButton {config} />
        <NodeStatusButton />
        <InboxButton {notificationCount} />
      </div>
    </div>
  </div>
</div>
