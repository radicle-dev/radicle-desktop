<script lang="ts">
  import type { Issue } from "@bindings/cob/issue/Issue";
  import type { IssueStatus } from "@app/views/repo/router";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import fuzzysort from "fuzzysort";

  import * as router from "@app/lib/router";
  import { modifierKey } from "@app/lib/utils";

  import Border from "./Border.svelte";
  import Icon from "./Icon.svelte";
  import IssueStateFilterButton from "./IssueStateFilterButton.svelte";
  import IssueTeaser from "@app/components/IssueTeaser.svelte";
  import NakedButton from "./NakedButton.svelte";
  import OutlineButton from "./OutlineButton.svelte";
  import TextInput from "./TextInput.svelte";

  const activeRouteStore = router.activeRouteStore;

  interface Props {
    changeFilter: (status: IssueStatus) => void;
    issues: Issue[];
    repo: RepoInfo;
    selectedIssueId?: string;
    status: IssueStatus;
  }

  const { changeFilter, issues, repo, selectedIssueId, status }: Props =
    $props();

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
      keys: ["issue.title", "labels", "assignees", "author", "issue.id"],
      threshold: 0.5,
      all: true,
    }),
  );
</script>

<style>
  .container {
    display: flex;
    align-items: center;
    min-height: 2.5rem;
    margin-bottom: 1rem;
    min-width: 28rem;
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
</style>

<div class="container">
  <div class="global-flex">
    <IssueStateFilterButton
      {status}
      counters={project.meta.issues}
      {changeFilter} />
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
  </div>
  <div class="global-flex" style:margin-left="auto">
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
      <Icon name="add" />New issue
    </OutlineButton>
  </div>
</div>

{#if showFilters}
  <div class="global-flex" style:margin="1rem 0">
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

{#if searchResults.length > 0}
  <div class="list">
    {#each searchResults as result}
      <IssueTeaser
        selected={result.obj.issue.id === selectedIssueId}
        focussed={searchResults.length === 1 && searchInput !== ""}
        compact
        issue={result.obj.issue}
        {status}
        rid={repo.rid} />
    {/each}
  </div>
{/if}

{#if searchResults.length === 0}
  <Border
    variant="ghost"
    styleFlexDirection="column"
    styleOverflow="hidden"
    styleAlignItems="center"
    styleJustifyContent="center">
    <div class="global-flex" style:height="84px" style:justify-content="center">
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
