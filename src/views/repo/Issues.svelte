<script lang="ts">
  import type { Config } from "@bindings/config/Config";
  import type { Issue } from "@bindings/cob/issue/Issue";
  import type { IssueStatus } from "./router";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import * as router from "@app/lib/router";

  import Layout from "./Layout.svelte";

  import Border from "@app/components/Border.svelte";
  import Button from "@app/components/Button.svelte";
  import CopyableId from "@app/components/CopyableId.svelte";
  import Icon from "@app/components/Icon.svelte";
  import IssueTeaser from "@app/components/IssueTeaser.svelte";
  import IssuesSecondColumn from "@app/components/IssuesSecondColumn.svelte";
  import Sidebar from "@app/components/Sidebar.svelte";

  interface Props {
    repo: RepoInfo;
    issues: Issue[];
    config: Config;
    status: IssueStatus;
  }

  const { repo, issues, config, status }: Props = $props();
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
    justify-content: space-between;
    min-height: 40px;
    margin-bottom: 0.5rem;
  }
</style>

<Layout
  hideSidebar
  styleSecondColumnOverflow="visible"
  publicKey={config.publicKey}>
  {#snippet headerCenter()}
    <CopyableId id={repo.rid} />
  {/snippet}

  {#snippet sidebar()}
    <Sidebar activeTab={{ type: "issues", status }} rid={repo.rid} />
  {/snippet}

  {#snippet secondColumn()}
    <IssuesSecondColumn {status} {repo} />
  {/snippet}

  <div class="container">
    <div class="header">
      <div>Issues</div>
      <div class="txt-regular txt-semibold">
        <Button
          variant="secondary"
          onclick={() => {
            void router.push({
              resource: "repo.createIssue",
              status,
              rid: repo.rid,
            });
          }}>
          <Icon name="plus" />New
        </Button>
      </div>
    </div>

    <div class="list">
      {#each issues as issue}
        <IssueTeaser {issue} rid={repo.rid} {status} />
      {/each}

      {#if issues.length === 0}
        <Border
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
  </div>
</Layout>
