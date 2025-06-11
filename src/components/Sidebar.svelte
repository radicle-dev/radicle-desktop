<script lang="ts">
  import * as router from "@app/lib/router";

  import { storeLayout, getLayout } from "@app/views/repo/Layout.svelte";

  import Icon from "@app/components/Icon.svelte";
  import Settings from "@app/components/Settings.svelte";
  import Border from "./Border.svelte";
  import NakedButton from "./NakedButton.svelte";

  interface Props {
    activeTab: "issues" | "patches";
    rid: string;
  }

  const { activeTab, rid }: Props = $props();
</script>

<style>
  .sidebar-button {
    cursor: pointer;
    border: 0;
    background: none;
    height: 2.5rem;
    width: 2.5rem;
    clip-path: var(--2px-corner-fill);
    margin: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-foreground-contrast);
    background-color: var(--color-background-float);
  }

  .sidebar-button:hover {
    background-color: var(--color-fill-float-hover);
  }
</style>

<div class="global-flex" style:flex-direction="column" style:gap="0.5rem">
  <div class="global-flex" style:height="2.5rem">
    <button
      class="sidebar-button"
      onclick={() => {
        void router.push({
          resource: "repo.home",
          rid,
        });
      }}>
      <Icon name="repo" />
    </button>
  </div>
  {#if activeTab === "issues"}
    <Border
      styleCursor="pointer"
      onclick={() => {
        void router.push({
          resource: "repo.issues",
          rid,
          status: "open",
        });
      }}
      variant="ghost"
      styleWidth="2.5rem"
      styleHeight="2.5rem"
      styleJustifyContent="center">
      <Icon name="issue" />
    </Border>
  {:else}
    <button
      class="sidebar-button"
      onclick={() => {
        void router.push({
          resource: "repo.issues",
          rid,
          status: "open",
        });
      }}>
      <Icon name="issue" />
    </button>
  {/if}

  {#if activeTab === "patches"}
    <Border
      styleCursor="pointer"
      onclick={() => {
        void router.push({
          resource: "repo.patches",
          rid,
          status: "open",
        });
      }}
      variant="ghost"
      styleWidth="2.5rem"
      styleHeight="2.5rem"
      styleJustifyContent="center">
      <Icon name="patch" />
    </Border>
  {:else}
    <button
      class="sidebar-button"
      onclick={() => {
        void router.push({
          resource: "repo.patches",
          rid,
          status: "open",
        });
      }}>
      <Icon name="patch" />
    </button>
  {/if}
</div>

<div style:z-index="20">
  <NakedButton
    styleHeight="2.5rem"
    variant="ghost"
    onclick={() => {
      if (getLayout()) {
        storeLayout("two-column");
      } else {
        storeLayout("one-column");
      }
    }}>
    {#if getLayout()}
      <Icon name="expand-panel" />
    {:else}
      <Icon name="collapse-panel" />
    {/if}
  </NakedButton>

  <Settings
    popoverProps={{
      popoverPositionBottom: "3rem",
      popoverPositionLeft: "0",
    }} />
</div>
