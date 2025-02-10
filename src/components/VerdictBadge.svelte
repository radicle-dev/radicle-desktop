<script lang="ts">
  import type { Review } from "@bindings/cob/patch/Review";
  import type { Snippet } from "svelte";

  import capitalize from "lodash/capitalize.js";
  import { verdictIcon, verdictIconColor } from "@app/lib/utils";

  import Icon from "@app/components/Icon.svelte";

  interface Props {
    children?: Snippet;
    verdict: Review["verdict"];
    hoverable?: boolean;
  }

  const { children, verdict, hoverable = false }: Props = $props();
</script>

<style>
  .badge {
    gap: 6px;
    padding-right: 10px;
  }
  .no-verdict {
    background-color: var(--color-fill-ghost);
  }
  .no-verdict.hoverable:hover {
    background-color: var(--color-fill-ghost-hover);
  }

  .accepted {
    background-color: var(--color-fill-diff-green-light);
  }
  .accepted.hoverable:hover {
    background-color: var(--color-fill-diff-green);
  }

  .rejected {
    background-color: var(--color-fill-diff-red-light);
  }
  .rejected.hoverable:hover {
    background-color: var(--color-fill-diff-red);
  }
</style>

<span
  class="global-counter badge"
  style:color={verdictIconColor(verdict)}
  class:hoverable
  class:no-verdict={verdict === undefined}
  class:accepted={verdict === "accept"}
  class:rejected={verdict === "reject"}>
  <Icon name={verdictIcon(verdict)} />
  {verdict ? capitalize(`${verdict}ed`) : "None"}
  {@render children?.()}
</span>
