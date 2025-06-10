<script lang="ts">
  import type { CacheEvent } from "@bindings/cob/CacheEvent";
  import type { Config } from "@bindings/config/Config";
  import type { Issue } from "@bindings/cob/issue/Issue";
  import type { IssueStatus } from "@app/views/repo/router";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import delay from "lodash/delay";
  import fuzzysort from "fuzzysort";
  import { Channel } from "@tauri-apps/api/core";

  import * as router from "@app/lib/router";
  import { explorerUrl, modifierKey } from "@app/lib/utils";
  import { invoke } from "@app/lib/invoke";
  import {
    issueCountMismatch,
    resetIssueCounts,
  } from "@app/lib/issueCounts.svelte";

  import Border from "@app/components/Border.svelte";
  import Button from "@app/components/Button.svelte";
  import Icon from "@app/components/Icon.svelte";
  import IssueTeaser from "@app/components/IssueTeaser.svelte";
  import IssuesSecondColumn from "@app/components/IssuesSecondColumn.svelte";
  import NodeBreadcrumb from "@app/components/NodeBreadcrumb.svelte";
  import Spinner from "@app/components/Spinner.svelte";
  import TextInput from "@app/components/TextInput.svelte";

  import BreadcrumbCopyButton from "./BreadcrumbCopyButton.svelte";
  import IssuesBreadcrumb from "./IssuesBreadcrumb.svelte";
  import Layout from "./Layout.svelte";
  import RepoBreadcrumb from "./RepoBreadcrumb.svelte";

  interface Props {
    repo: RepoInfo;
    issues: Issue[];
    config: Config;
    status: IssueStatus;
    notificationCount: number;
  }

  /* eslint-disable prefer-const */
  let { notificationCount, repo, issues, config, status }: Props = $props();
  /* eslint-enable prefer-const */

  let cacheState: CacheEvent | undefined = $state();

  const project = $derived(repo.payloads["xyz.radicle.project"]!);

  let searchInput = $state("");

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
</script>

<style>
  .container {
    padding: 1rem 1rem 1rem 0;
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .header {
    font-weight: var(--font-weight-medium);
    font-size: var(--font-size-medium);
    display: flex;
    align-items: center;
    min-height: 2.5rem;
    margin-bottom: 1rem;
  }
</style>

<Layout
  hideSidebar
  styleSecondColumnOverflow="visible"
  {config}
  {notificationCount}>
  {#snippet breadcrumbs()}
    <NodeBreadcrumb {config} />
    <Icon name="chevron-right" />
    <RepoBreadcrumb name={project.data.name} rid={repo.rid} />
    <Icon name="chevron-right" />
    <IssuesBreadcrumb rid={repo.rid} {status} />
    <BreadcrumbCopyButton
      url={explorerUrl(`${repo.rid}/issues`)}
      icon="repo"
      id={repo.rid} />
  {/snippet}

  {#snippet secondColumn()}
    <IssuesSecondColumn {status} {repo} />
  {/snippet}

  <div class="container">
    {#if issueCountMismatch(status)}
      <div style="margin-bottom: 1rem;">
        <Border
          styleOverflow="hidden"
          styleBackgroundColor="var(--color-fill-private)"
          stylePadding="0.25rem 0.5rem"
          styleGap="1rem"
          variant="outline">
          <div class="txt-overflow txt-small global-flex">
            <Icon name="warning" />
            <span class="txt-overflow">
              There’s a problem with your COB cache, so some issues may not be
              displayed. You can rebuild the cache to resolve this.
            </span>
          </div>
          <div style:margin-left="auto">
            <Button
              variant="ghost"
              onclick={rebuildIssueCache}
              disabled={cacheState !== undefined}>
              {#if cacheState?.event === "started" || cacheState?.event === "progress"}
                Rebuilding
                <Spinner />
              {:else if cacheState?.event === "finished"}
                Done
                <Icon name="checkmark" />
              {:else}
                Rebuild cache
              {/if}
            </Button>
          </div>
        </Border>
      </div>
    {/if}
    <div class="header">
      <div>Issues</div>
      <div class="global-flex" style:margin-left="auto" style:gap="0.75rem">
        {#if issues.length > 0}
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
              searchInput = "";
            }}
            placeholder={`Fuzzy filter issues ${modifierKey()} + f`}
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
        {/if}
        <Button
          styleHeight="2.5rem"
          variant="secondary"
          onclick={() => {
            void router.push({
              resource: "repo.createIssue",
              status,
              rid: repo.rid,
            });
          }}>
          <Icon name="add" />New issue
        </Button>
      </div>
    </div>

    <div class="list">
      {#each searchResults as result}
        <IssueTeaser
          focussed={searchResults.length === 1 && searchInput !== ""}
          issue={result.obj.issue}
          rid={repo.rid}
          {status} />
      {/each}

      {#if searchResults.length === 0}
        <Border
          variant="ghost"
          styleFlexDirection="column"
          styleAlignItems="center"
          styleJustifyContent="center">
          <div
            class="global-flex"
            style:height="5.25rem"
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
  </div>
</Layout>
