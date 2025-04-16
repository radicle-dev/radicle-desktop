<script lang="ts">
  import type { Review } from "@bindings/cob/patch/Review";
  import type { Snippet } from "svelte";

  import capitalize from "lodash/capitalize.js";
  import { verdictIcon } from "@app/lib/utils";

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
    gap: 0.375rem;
    padding-right: 0.625rem;
  }
  .no-verdict {
    background-color: var(--color-fill-ghost);
    color: var(--color-foreground-dim);
  }
  .no-verdict.hoverable:hover {
    background-color: var(--color-fill-ghost-hover);
  }

  .accepted {
    background-color: var(--color-fill-diff-green-light);
    color: var(--color-foreground-success);
  }
  .accepted.hoverable:hover {
    background-color: var(--color-fill-diff-green);
  }

  .rejected {
    background-color: var(--color-fill-diff-red-light);
    color: var(--color-foreground-red);
  }
  .rejected.hoverable:hover {
    background-color: var(--color-fill-diff-red);
  }
</style>

<span
  class="global-counter badge"
  class:hoverable
  class:no-verdict={verdict === undefined}
  class:accepted={verdict === "accept"}
  class:rejected={verdict === "reject"}>
  <Icon name={verdictIcon(verdict)} />
  {verdict ? capitalize(`${verdict}ed`) : "None"}
  {@render children?.()}
</span>
