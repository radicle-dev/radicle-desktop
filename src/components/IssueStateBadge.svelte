<script lang="ts">
  import type { Issue } from "@bindings/cob/issue/Issue";

  import capitalize from "lodash/capitalize";
  import { issueStatusColor } from "@app/lib/utils";

  export let state: Issue["state"];
</script>

<div
  class="global-counter txt-small"
  style:width="fit-content"
  style:color="var(--color-foreground-match-background)"
  style:background-color={issueStatusColor[state.status]}>
  {capitalize(state.status)}{state.status === "closed"
    ? ` as ${state.reason}`
    : ""}
</div>
