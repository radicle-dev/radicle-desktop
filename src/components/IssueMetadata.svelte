<script lang="ts">
  import type { Issue } from "@bindings/cob/issue/Issue";

  import { authorForNodeId } from "@app/lib/utils";

  import Border from "@app/components/Border.svelte";
  import IssueStateBadge from "@app/components/IssueStateBadge.svelte";
  import NodeId from "@app/components/NodeId.svelte";

  interface Props {
    issue: Issue;
  }

  const { issue }: Props = $props();
</script>

<style>
  .divider {
    width: 2px;
    background-color: var(--color-fill-ghost);
    height: calc(100% + 4px);
    top: 0;
    position: relative;
  }
  .section {
    padding: 0.5rem;
    font-size: var(--font-size-small);
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    height: 100%;
  }
  .section-title {
    margin-bottom: 0.5rem;
    color: var(--color-foreground-dim);
  }
</style>

<Border variant="ghost" styleGap="0">
  <div class="section" style:min-width="8rem">
    <div class="section-title">Status</div>
    <IssueStateBadge state={issue.state} />
  </div>

  <div class="divider"></div>

  <div class="section" style:flex="1">
    <div class="section-title">Labels</div>
    <div class="global-flex" style:flex-wrap="wrap">
      {#each issue.labels as label}
        <div class="global-counter txt-small">{label}</div>
      {:else}
        <span class="txt-missing">No labels.</span>
      {/each}
    </div>
  </div>

  <div class="divider"></div>

  <div class="section" style:flex="1">
    <div class="section-title">Assignees</div>
    <div class="global-flex" style:flex-wrap="wrap">
      {#each issue.assignees as assignee}
        <NodeId {...authorForNodeId(assignee)} />
      {:else}
        <span class="txt-missing">Not assigned to anyone.</span>
      {/each}
    </div>
  </div>
</Border>
