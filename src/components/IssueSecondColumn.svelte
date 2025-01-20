<script lang="ts">
  import type { Issue } from "@bindings/cob/issue/Issue";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";
  import type { IssueStatus } from "@app/views/repo/router";

  import capitalize from "lodash/capitalize";

  import * as router from "@app/lib/router";
  import { issueStatusColor } from "@app/lib/utils";

  import Border from "./Border.svelte";
  import DropdownList from "./DropdownList.svelte";
  import DropdownListItem from "./DropdownListItem.svelte";
  import Icon from "./Icon.svelte";
  import IssueTeaser from "@app/components/IssueTeaser.svelte";
  import OutlineButton from "./OutlineButton.svelte";
  import Popover, { closeFocused } from "./Popover.svelte";

  const activeRouteStore = router.activeRouteStore;

  interface Props {
    repo: RepoInfo;
    selectedIssueId?: string;
    issues: Issue[];
    status: IssueStatus;
    title: string;
    changeFilter: (status: IssueStatus) => void;
  }

  /* eslint-disable prefer-const */
  let { repo, selectedIssueId, issues, status, title, changeFilter }: Props =
    $props();
  /* eslint-enable prefer-const */

  const project = $derived(repo.payloads["xyz.radicle.project"]!);
</script>

<style>
  .container {
    display: flex;
    justify-content: space-between;
    align-items: center;
    min-height: 40px;
  }
  .issue-list {
    margin-top: 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding-bottom: 1rem;
  }
</style>

{#snippet icons(status: IssueStatus)}
  <div
    class="icon"
    style:color={status === "all" ? undefined : issueStatusColor[status]}>
    <Icon name={status === "closed" ? "issue-closed" : "issue"} />
  </div>
{/snippet}

{#snippet counters(status: IssueStatus)}
  <div style:margin-left="auto" style:padding-left="0.25rem">
    {#if status === "all"}
      {project.meta.issues.open + project.meta.issues.closed}
    {:else}
      {project.meta.issues[status]}
    {/if}
  </div>
{/snippet}

<div class="container">
  <div
    class="txt-regular txt-semibold global-flex"
    style:gap="4px"
    style:white-space="nowrap">
    {title}
    <Icon name="chevron-right" />
    Issues
  </div>

  <div class="global-flex" style:margin-left="1rem">
    <Popover popoverPositionRight="0" popoverPositionTop="2.5rem">
      {#snippet toggle(onclick)}
        <OutlineButton variant="ghost" {onclick}>
          {@render icons(status)}
          {capitalize(status)}
          {@render counters(status)}
          <Icon name="chevron-down" />
        </OutlineButton>
      {/snippet}

      {#snippet popover()}
        <Border variant="ghost">
          <DropdownList items={["all", "open", "closed"] as IssueStatus[]}>
            {#snippet item(state)}
              <DropdownListItem
                style="gap: 0.5rem"
                selected={status === state}
                onclick={() => {
                  changeFilter(state);
                  closeFocused();
                }}>
                {@render icons(state)}
                {capitalize(state)}
                {@render counters(state)}
              </DropdownListItem>
            {/snippet}
          </DropdownList>
        </Border>
      {/snippet}
    </Popover>

    <OutlineButton
      variant="ghost"
      disabled={$activeRouteStore.resource === "repo.createIssue"}
      onclick={() => {
        void router.push({
          resource: "repo.createIssue",
          rid: repo.rid,
          status,
        });
      }}>
      <Icon name="plus" />New
    </OutlineButton>
  </div>
</div>
<div class="issue-list">
  {#each issues as issue}
    <IssueTeaser
      compact
      {issue}
      {status}
      rid={repo.rid}
      selected={issue.id === selectedIssueId} />
  {/each}

  {#if issues.length === 0}
    <Border
      styleMinWidth="25rem"
      variant="ghost"
      styleAlignItems="center"
      styleJustifyContent="center">
      <div
        class="global-flex"
        style:height="74px"
        style:justify-content="center">
        <div class="txt-missing txt-small global-flex" style:gap="0.25rem">
          <Icon name="none" />
          {#if status === "all"}
            No issues.
          {:else}
            No {status} issues.
          {/if}
        </div>
      </div>
    </Border>
  {/if}
</div>
