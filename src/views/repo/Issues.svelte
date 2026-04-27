<script lang="ts">
  import type { IssueStatus } from "@app/views/repo/router";
  import type { CacheEvent } from "@bindings/cob/CacheEvent";
  import type { Issue } from "@bindings/cob/issue/Issue";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import { Channel } from "@tauri-apps/api/core";
  import fuzzysort from "fuzzysort";
  import delay from "lodash/delay";

  import { invoke } from "@app/lib/invoke";
  import {
    issueCountMismatch,
    resetIssueCounts,
  } from "@app/lib/issueCounts.svelte";
  import { show } from "@app/lib/modal";
  import * as router from "@app/lib/router";
  import { modifierKey } from "@app/lib/utils";

  import Button from "@app/components/Button.svelte";
  import CobCacheWarning from "@app/components/CobCacheWarning.svelte";
  import FuzzySearch from "@app/components/FuzzySearch.svelte";
  import Icon from "@app/components/Icon.svelte";
  import IssueTeaser from "@app/components/IssueTeaser.svelte";
  import ScrollArea from "@app/components/ScrollArea.svelte";
  import Topbar from "@app/components/Topbar.svelte";
  import CreateIssueModal from "@app/modals/CreateIssue.svelte";

  import Layout from "./Layout.svelte";

  interface Props {
    repo: RepoInfo;
    issues: Issue[];
    status: IssueStatus;
  }

  /* eslint-disable prefer-const */
  let { repo, issues, status }: Props = $props();
  /* eslint-enable prefer-const */

  let cacheState: CacheEvent | undefined = $state();

  let searchInput = $state("");
  let showSearch = $state(false);

  async function rebuildIssueCache() {
    try {
      await invoke("rebuild_issue_cache", {
        rid: repo.rid,
        onEvent: new Channel<CacheEvent>(message => {
          cacheState = message;
        }),
      });
    } catch (error) {
      console.error(error);
    } finally {
      issues = await invoke<Issue[]>("list_issues", { rid: repo.rid, status });

      resetIssueCounts();

      delay(() => {
        cacheState = undefined;
      }, 1500);
    }
  }

  $effect(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    status;

    searchInput = "";
    showSearch = false;
  });

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

  const project = $derived(repo.payloads["xyz.radicle.project"]!);
</script>

<style>
  .page {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  .topbar-title {
    font: var(--txt-body-m-semibold);
    color: var(--color-text-secondary);
    padding-right: 0.25rem;
  }
  .filters {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }
  .filter {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
    padding: 0.25rem 0.5rem;
    border-radius: var(--border-radius-sm);
    text-decoration: none;
    cursor: pointer;
    white-space: nowrap;
  }
  .filter:hover {
    background-color: var(--color-surface-subtle);
    color: var(--color-text-primary);
  }
  .filter.active {
    background-color: var(--color-surface-subtle);
  }
  .filter .global-counter-badge {
    margin-left: 0.25rem;
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-height: 100%;
  }
</style>

<Layout selfScroll>
  <div class="page">
    <Topbar>
      <span class="topbar-title">Issues</span>
      <div class="filters">
        <a
          class="filter"
          class:active={status === "all"}
          href={router.routeToPath({
            resource: "repo.issues",
            rid: repo.rid,
            status: "all",
          })}>
          <Icon name="issue" />All
          <span class="global-counter-badge">
            {project.meta.issues.open + project.meta.issues.closed}
          </span>
        </a>
        <a
          class="filter"
          class:active={status === "open"}
          href={router.routeToPath({
            resource: "repo.issues",
            rid: repo.rid,
            status: "open",
          })}>
          <Icon name="issue" />Open
          <span class="global-counter-badge">{project.meta.issues.open}</span>
        </a>
        <a
          class="filter"
          class:active={status === "closed"}
          href={router.routeToPath({
            resource: "repo.issues",
            rid: repo.rid,
            status: "closed",
          })}>
          <Icon name="issue-closed" />Closed
          <span class="global-counter-badge">{project.meta.issues.closed}</span>
        </a>
      </div>
      <div class="global-flex" style:margin-left="auto" style:gap="0.5rem">
        <FuzzySearch
          hasItems={issues.length > 0}
          placeholder={`Fuzzy filter issues ${modifierKey()} + f`}
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
          bind:show={showSearch}
          bind:value={searchInput} />
        <Button
          styleHeight="2rem"
          variant="secondary"
          onclick={() =>
            show({
              component: CreateIssueModal,
              props: { repo },
            })}>
          <Icon name="plus" />New issue
        </Button>
      </div>
    </Topbar>

    <ScrollArea style="height: 100%; min-width: 0;">
      {#if issueCountMismatch(status)}
        <CobCacheWarning
          noun="issues"
          {cacheState}
          onRebuild={rebuildIssueCache} />
      {/if}

      <div class="list">
        {#each searchResults as result}
          <IssueTeaser
            focussed={searchResults.length === 1 && searchInput !== ""}
            issue={result.obj.issue}
            rid={repo.rid}
            {status} />
        {/each}

        {#if searchResults.length === 0}
          <div
            class="global-flex"
            style:flex="1"
            style:justify-content="center"
            style:align-items="center">
            <div
              class="txt-missing txt-body-m-regular global-flex"
              style:gap="0.25rem">
              {#if issues.length > 0 && searchResults.length === 0}
                No matching issues
              {:else}
                No {status === "all" ? "" : status} issues
              {/if}
            </div>
          </div>
        {/if}
      </div>
    </ScrollArea>
  </div>
</Layout>
