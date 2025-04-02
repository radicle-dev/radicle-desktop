<script lang="ts">
  import type { Issue } from "@bindings/cob/issue/Issue";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";
  import type { IssueStatus } from "@app/views/repo/router";

  import capitalize from "lodash/capitalize";
  import fuzzysort from "fuzzysort";

  import * as router from "@app/lib/router";
  import { issueStatusColor, modifierKey } from "@app/lib/utils";

  import Border from "./Border.svelte";
  import DropdownList from "./DropdownList.svelte";
  import DropdownListItem from "./DropdownListItem.svelte";
  import Icon from "./Icon.svelte";
  import IssueTeaser from "@app/components/IssueTeaser.svelte";
  import Link from "./Link.svelte";
  import NakedButton from "./NakedButton.svelte";
  import OutlineButton from "./OutlineButton.svelte";
  import Popover, { closeFocused } from "./Popover.svelte";
  import TextInput from "./TextInput.svelte";

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

  let showFilters: boolean = $state(false);
  let searchInput = $state("");

  const searchableIssues = $derived(
    issues
      .flatMap(i => {
        return {
          issue: i,
          labels: i.labels.join(" "),
          assignees: i.assignees
            .map(a => {
              return a.alias ?? "";
            })
            .join(" "),
          author: i.author.alias ?? "",
        };
      })
      .filter((item): item is NonNullable<typeof item> => item !== undefined),
  );

  const searchResults = $derived(
    fuzzysort.go(searchInput, searchableIssues, {
      keys: ["issue.title", "labels", "assignees", "author"],
      all: true,
    }),
  );
</script>

<style>
  .container {
    display: flex;
    align-items: center;
    min-height: 2.5rem;
  }
  .issue-list {
    margin-top: 1rem;
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
    class="txt-medium global-flex"
    style:font-weight="var(--font-weight-medium)"
    style:gap="4px"
    style:white-space="nowrap">
    {title}
    <Icon name="chevron-right" />
    <Link
      underline={false}
      route={{
        resource: "repo.issues",
        rid: repo.rid,
        status: "open",
      }}>
      Issues
    </Link>
  </div>

  <div class="global-flex" style:margin-left="auto">
    <NakedButton
      styleHeight="2.5rem"
      keyShortcuts="ctrl+f"
      variant="ghost"
      active={showFilters}
      onclick={() => {
        if (showFilters) {
          showFilters = false;
          searchInput = "";
        } else {
          showFilters = true;
        }
      }}>
      <Icon name="filter" />
    </NakedButton>

    <OutlineButton
      variant="ghost"
      styleHeight="2.5rem"
      active={$activeRouteStore.resource === "repo.createIssue"}
      onclick={() => {
        if ($activeRouteStore.resource === "repo.createIssue") {
          window.history.back();
        } else {
          void router.push({
            resource: "repo.createIssue",
            rid: repo.rid,
            status,
          });
        }
      }}>
      <Icon name="plus" />New
    </OutlineButton>
  </div>
</div>

{#if showFilters}
  <div class="global-flex" style:margin="1rem 0">
    <Popover popoverPositionLeft="0" popoverPositionTop="3rem">
      {#snippet toggle(onclick)}
        <OutlineButton variant="ghost" {onclick} styleHeight="2.5rem">
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
                styleGap="0.5rem"
                styleMinHeight="2.5rem"
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
    <TextInput
      onSubmit={async () => {
        if (searchResults.length === 1) {
          await router.push({
            resource: "repo.issue",
            rid: repo.rid,
            issue: searchResults[0].obj.issue.id,
            status,
          });
        }
      }}
      onDismiss={() => {
        showFilters = false;
        searchInput = "";
      }}
      placeholder={`Fuzzy filter issues ${modifierKey()} + f`}
      autofocus
      keyShortcuts="ctrl+f"
      bind:value={searchInput}>
      {#snippet left()}
        <div
          style:color="var(--color-foreground-dim)"
          style:padding-left="0.5rem">
          <Icon name="filter" />
        </div>
      {/snippet}
    </TextInput>
  </div>
{/if}

<div class="issue-list">
  {#each searchResults as result}
    <IssueTeaser
      focussed={searchResults.length === 1 && searchInput !== ""}
      compact
      issue={result.obj.issue}
      {status}
      rid={repo.rid}
      selected={result.obj.issue.id === selectedIssueId} />
  {/each}

  {#if searchResults.length === 0}
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
          {#if issues.length > 0 && searchResults.length === 0}
            No matching issues.
          {:else}
            No {status === "all" ? "" : status} issues.
          {/if}
        </div>
      </div>
    </Border>
  {/if}
</div>
