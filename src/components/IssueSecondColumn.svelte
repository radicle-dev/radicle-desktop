<script lang="ts">
  import type { Issue } from "@bindings/cob/issue/Issue";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import * as router from "@app/lib/router";

  import IssueTeaser from "@app/components/IssueTeaser.svelte";
  import Icon from "./Icon.svelte";
  import OutlineButton from "./OutlineButton.svelte";

  interface Props {
    repo: RepoInfo;
    selectedIssueId?: string;
    issues: Issue[];
  }

  /* eslint-disable prefer-const */
  let { repo, selectedIssueId, issues }: Props = $props();
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
  <div class="txt-medium" style:font-weight="var(--font-weight-medium)">
    Issues
  </div>

  <OutlineButton
    variant="ghost"
    onclick={() => {
      void router.push({
        resource: "repo.createIssue",
        rid: repo.rid,
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
      rid={repo.rid}
      selected={issue.id === selectedIssueId} />
  {/each}
</div>
