<script lang="ts">
  import * as router from "@app/lib/router";

  import Icon from "@app/components/Icon.svelte";
  import Settings from "@app/components/Settings.svelte";
  import Border from "./Border.svelte";

  interface Props {
    activeTab: "issues" | "patches";
    rid: string;
    activeIconColor?: string;
  }

  const { activeTab, rid, activeIconColor }: Props = $props();
</script>

<style>
  .sidebar-button {
    cursor: pointer;
    border: 0;
    background: none;
    height: 40px;
    width: 40px;
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
  <div class="global-flex" style:margin-bottom="5px" style:height="40px">
    <Icon name="repo" />
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
      styleWidth="40px"
      styleHeight="40px"
      styleJustifyContent="center">
      <div style:color={activeIconColor}><Icon name="issue" /></div>
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
      onclick={() => {
        void router.push({
          resource: "repo.patches",
          rid,
          status: "open",
        });
      }}
      variant="ghost"
      styleWidth="40px"
      styleHeight="40px"
      styleJustifyContent="center">
      <div style:color={activeIconColor}><Icon name="patch" /></div>
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

<Settings
  popoverProps={{
    popoverPositionBottom: "3rem",
    popoverPositionLeft: "0",
  }} />
