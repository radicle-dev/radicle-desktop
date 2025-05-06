<script lang="ts">
  import type { Config } from "@bindings/config/Config";
  import type { Issue } from "@bindings/cob/issue/Issue";
  import type { IssueStatus } from "./router";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import fuzzysort from "fuzzysort";

  import * as router from "@app/lib/router";
  import { modifierKey } from "@app/lib/utils";

  import Layout from "./Layout.svelte";

  import Border from "@app/components/Border.svelte";
  import Button from "@app/components/Button.svelte";
  import CopyableId from "@app/components/CopyableId.svelte";
  import Icon from "@app/components/Icon.svelte";
  import IssueTeaser from "@app/components/IssueTeaser.svelte";
  import IssuesSecondColumn from "@app/components/IssuesSecondColumn.svelte";
  import TextInput from "@app/components/TextInput.svelte";

  interface Props {
    repo: RepoInfo;
    issues: Issue[];
    config: Config;
    status: IssueStatus;
  }

  /* eslint-disable prefer-const */
  let { repo, issues, config, status }: Props = $props();
  /* eslint-enable prefer-const */

  let searchInput = $state("");

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
      keys: ["issue.title", "labels", "assignees", "author"],
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
  publicKey={config.publicKey}>
  {#snippet headerCenter()}
    <CopyableId id={repo.rid} />
  {/snippet}

  {#snippet secondColumn()}
    <IssuesSecondColumn {status} {repo} />
  {/snippet}

  <div class="container">
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
          <Icon name="add" />New
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
