<script lang="ts">
  import type { IssueOps } from "@bindings/IssueOps";

  import { authorForNodeId, formatTimestamp } from "@app/lib/utils";

  import Border from "@app/components/Border.svelte";
  import IssueStateBadge from "@app/components/IssueStateBadge.svelte";
  import NodeId from "@app/components/NodeId.svelte";

  export let issueOp: IssueOps;
</script>

<!-- TODO use narrower type to get rid if this conditional.-->
{#if issueOp.action.type === "lifecycle"}
  <Border variant="float" stylePadding="1rem">
    <div class="txt-small">
      <div class="global-flex txt-small">
        <NodeId {...authorForNodeId(issueOp.author)} />
        changed status to
        <IssueStateBadge state={issueOp.action.state} />
        {formatTimestamp(issueOp.timestamp)}
      </div>
    </div>
  </Border>
{/if}
