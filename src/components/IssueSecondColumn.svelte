<script lang="ts">
  import type { Issue } from "@bindings/cob/issue/Issue";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";
  import type { IssueStatus } from "@app/views/repo/router";

  import * as router from "@app/lib/router";

  import IssueTeaser from "@app/components/IssueTeaser.svelte";
  import Icon from "./Icon.svelte";
  import OutlineButton from "./OutlineButton.svelte";

  const activeRouteStore = router.activeRouteStore;

  interface Props {
    repo: RepoInfo;
    selectedIssueId?: string;
    issues: Issue[];
    status: IssueStatus;
    title: string;
  }

  /* eslint-disable prefer-const */
  let { repo, selectedIssueId, issues, status, title }: Props = $props();
  /* eslint-enable prefer-const */
</script>

<style>
  .issue-list {
    margin-top: 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding-bottom: 1rem;
  }
</style>

<div class="global-flex" style:justify-content="space-between">
  <div class="txt-regular txt-semibold global-flex" style:gap="4px">
    {title}
    <Icon name="chevron-right" />
    Issues
  </div>

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
<div class="issue-list">
  {#each issues as issue}
    <IssueTeaser
      compact
      {issue}
      {status}
      rid={repo.rid}
      selected={issue.id === selectedIssueId} />
  {/each}
</div>
