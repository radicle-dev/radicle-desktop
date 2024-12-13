<script lang="ts">
  import type { Patch } from "@bindings/cob/patch/Patch";

  import capitalize from "lodash/capitalize";
  import { patchStatusColor } from "@app/lib/utils";

  interface Props {
    state: Patch["state"];
  }

  const { state }: Props = $props();
</script>

<div
  class="global-counter txt-small"
  style:width="fit-content"
  style:color="var(--color-foreground-match-background)"
  style:background-color={patchStatusColor[state.status]}>
  {capitalize(state.status)}
</div>
