<script lang="ts">
  import type { Operation } from "@bindings/cob/Operation";
  import type { Action } from "@bindings/cob/patch/Action";

  import { authorForNodeId, formatTimestamp } from "@app/lib/utils";

  import Border from "@app/components/Border.svelte";
  import NodeId from "@app/components/NodeId.svelte";
  import PatchStateBadge from "./PatchStateBadge.svelte";

  interface Props {
    action: Extract<Action, { type: "lifecycle" }>;
    op: Operation<Action>;
  }

  const { op, action }: Props = $props();
</script>

<Border variant="float" stylePadding="1rem">
  <div class="txt-small">
    <div class="global-flex txt-small">
      <NodeId {...authorForNodeId(op.author)} />
      changed status to
      <PatchStateBadge state={action.state} />
      {formatTimestamp(op.timestamp)}
    </div>
  </div>
</Border>
