<script lang="ts">
  import type { Operation } from "@bindings/cob/issue/Operation";

  import { authorForNodeId, formatTimestamp } from "@app/lib/utils";

  import Border from "@app/components/Border.svelte";
  import IssueStateBadge from "@app/components/IssueStateBadge.svelte";
  import NodeId from "@app/components/NodeId.svelte";

  interface Props {
    operation: Extract<Operation, { type: "lifecycle" }>;
  }

  const { operation }: Props = $props();
</script>

<Border variant="float" stylePadding="1rem">
  <div class="txt-small">
    <div class="global-flex txt-small">
      <NodeId {...authorForNodeId(operation.author)} />
      changed status to
      <IssueStateBadge state={operation.state} />
      {formatTimestamp(operation.timestamp)}
    </div>
  </div>
</Border>
