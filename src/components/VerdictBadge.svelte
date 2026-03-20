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
    width: fit-content;
    font: var(--txt-body-m-regular);
  }
  .no-verdict {
    background-color: var(--color-surface-subtle);
    color: var(--color-text-secondary);
  }
  .no-verdict.hoverable:hover {
    background-color: var(--color-surface-mid);
  }

  .accepted {
    background-color: var(--color-feedback-success-bg);
    color: var(--color-feedback-success-text);
  }
  .accepted.hoverable:hover {
    background-color: var(--color-feedback-success-bg);
  }

  .rejected {
    background-color: var(--color-feedback-error-bg);
    color: var(--color-feedback-error-text);
  }
  .rejected.hoverable:hover {
    background-color: var(--color-feedback-error-bg);
  }
</style>

<span
  class="global-chip badge"
  class:hoverable
  class:no-verdict={verdict === undefined}
  class:accepted={verdict === "accept"}
  class:rejected={verdict === "reject"}>
  <Icon name={verdictIcon(verdict)} />
  {verdict ? capitalize(`${verdict}ed`) : "None"}
  {@render children?.()}
</span>
