<script lang="ts">
  import type { RepoCount } from "@bindings/repo/RepoCount";
  import type { HomeReposTab } from "@app/views/home/router";

  import * as router from "@app/lib/router";

  import Border from "./Border.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Settings from "@app/components/Settings.svelte";

  interface Props {
    activeTab: HomeReposTab;
    repoCount: RepoCount;
  }

  const { activeTab, repoCount }: Props = $props();
</script>

<style>
  .container {
    display: flex;
    flex-direction: column;
    height: 100%;
    justify-content: space-between;
  }
  .tab {
    align-items: center;
    gap: 0.5rem;
    background-color: var(--color-background-float);
    clip-path: var(--1px-corner-fill);
    display: flex;
    font-size: var(--font-size-small);
    justify-content: space-between;
    padding: 0.5rem 0.25rem 0.5rem 0.5rem;
    width: 100%;
  }
  .tab:not(.active) {
    color: var(--color-foreground-dim);
  }
  .tab:not(.active):hover {
    background-color: var(--color-fill-float-hover);
  }
  .active {
    background-color: var(--color-background-default);
    font-weight: var(--font-weight-semibold);
  }
</style>

<div class="container">
  <div>
    <Border
      styleCursor="pointer"
      variant="ghost"
      styleFlexDirection="column"
      styleGap="2px"
      styleBackgroundColor="var(--color-background-float)">
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="tab txt-small"
        class:active={activeTab === "all"}
        onclick={() => router.push({ resource: "home", activeTab: "all" })}>
        <div class="global-flex"><Icon name="repo" />Repositories</div>
        <div class="global-counter">
          {repoCount.total}
        </div>
      </div>
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="tab"
        class:active={activeTab === "delegate"}
        onclick={() =>
          router.push({
            resource: "home",
            activeTab: "delegate",
          })}>
        <div class="global-flex">
          <Icon name="delegate" />
          <div>Delegate</div>
        </div>
        <div class="global-counter">{repoCount.delegate}</div>
      </div>
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="tab"
        class:active={activeTab === "contributor"}
        onclick={() =>
          router.push({
            resource: "home",
            activeTab: "contributor",
          })}>
        <div class="global-flex">
          <Icon name="user" />
          <div>Contributor</div>
        </div>
        <div class="global-counter">{repoCount.contributor}</div>
      </div>
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="tab"
        class:active={activeTab === "private"}
        onclick={() =>
          router.push({
            resource: "home",
            activeTab: "private",
          })}>
        <div class="global-flex">
          <Icon name="lock" />
          <div>Private</div>
        </div>
        <div class="global-counter">{repoCount.private}</div>
      </div>
    </Border>
  </div>

  <Settings
    compact={false}
    popoverProps={{
      popoverPositionBottom: "3rem",
      popoverPositionLeft: "0",
    }} />
</div>
