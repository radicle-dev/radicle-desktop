<script lang="ts">
  import type { HomeInboxTab, HomeReposTab } from "@app/lib/router/definitions";
  import type { NotificationCount } from "@bindings/cob/inbox/NotificationCount";
  import type { RepoCount } from "@bindings/repo/RepoCount";

  import sum from "lodash/sum";

  import * as router from "@app/lib/router";
  import Border from "./Border.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Link from "./Link.svelte";
  import Settings from "@app/components/Settings.svelte";

  interface Props {
    activeTab:
      | { type: "inbox"; repo?: HomeInboxTab }
      | { type: "repos"; filter?: HomeReposTab };
    notificationCount: Map<string, NotificationCount>;
    repoCount: RepoCount;
  }

  const { notificationCount, repoCount, activeTab }: Props = $props();
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
    padding: 8px 4px 8px 8px;
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
  .highlight {
    color: var(--color-foreground-contrast);
  }
</style>

<div class="container">
  <div>
    <div style:margin-bottom="0.5rem">
      {#if activeTab.type === "inbox"}
        <Border
          styleCursor="pointer"
          variant="ghost"
          styleFlexDirection="column"
          styleGap="2px"
          styleBackgroundColor="var(--color-background-float)">
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            class="tab"
            class:active={!activeTab.repo}
            onclick={() => router.push({ resource: "inbox" })}>
            <div class="global-flex"><Icon name="inbox" />Inbox</div>
            <div class="global-counter">
              {sum(Array.from(notificationCount.values()).map(c => c.count))}
            </div>
          </div>
          {#each notificationCount.entries() as [_, n]}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="tab"
              onclick={() =>
                router.push({
                  resource: "inbox",
                  activeTab: n,
                })}
              class:active={activeTab.repo?.rid === n.rid}>
              <div class="global-flex">
                <Icon name="repo" />{n.name}
              </div>
              <div
                class="global-counter"
                class:highlight={activeTab.repo?.rid === n.rid}>
                {n.count}
              </div>
            </div>
          {/each}
        </Border>
      {:else}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <div
          class="tab"
          style:cursor="pointer"
          onclick={() => router.push({ resource: "inbox" })}
          style:color="var(--color-foreground-contrast)"
          style:padding-left="12px">
          <div class="global-flex"><Icon name="inbox" />Inbox</div>
          <div class="global-counter">
            {sum(Array.from(notificationCount.values()).map(c => c.count))}
          </div>
        </div>
      {/if}
    </div>

    {#if activeTab.type === "repos"}
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
          class:active={!activeTab.filter}
          onclick={() => router.push({ resource: "home" })}>
          <div class="global-flex"><Icon name="repo" />Repositories</div>
          <div class="global-counter">
            {repoCount.total}
          </div>
        </div>
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="tab"
          class:active={activeTab.filter === "delegate"}
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
          class:active={activeTab.filter === "private"}
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
    {:else}
      <Border
        styleBackgroundColor="var(--color-background-float)"
        variant="float">
        <Link
          styleWidth="100%"
          underline={false}
          route={{
            resource: "home",
          }}>
          <div
            style:justify-content="space-between"
            style:width="100%"
            class="tab">
            <div class="global-flex">
              <Icon name="repo" />
              Repositories
            </div>
            <div class="global-counter">
              {repoCount.total}
            </div>
          </div>
        </Link>
      </Border>
    {/if}
  </div>

  <Settings
    compact={false}
    popoverProps={{
      popoverPositionBottom: "3rem",
      popoverPositionLeft: "0",
    }} />
</div>
