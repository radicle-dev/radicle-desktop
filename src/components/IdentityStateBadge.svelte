<script lang="ts" module>
  import type { State } from "@bindings/identity/State";

  import { formatOid } from "@app/lib/utils";

  const icons = {
    active: "hourglass",
    accepted: "checkmark",
    rejected: "close",
  } as const;

  const labels = {
    active: "Active",
    accepted: "Accepted",
    rejected: "Rejected",
  };

  export function stateIcon(state: State): (typeof icons)[State["status"]] {
    return icons[state.status];
  }

  export function stateCaption(state: State): string | undefined {
    if (state.status === "rejected") {
      if (state.reason.type === "vote") return "Rejected by delegate votes";
      if (state.reason.type === "parent")
        return "Rejected because its parent revision was rejected";
      return `Rejected because sibling ${formatOid(state.reason.revision)} was accepted`;
    }
    return undefined;
  }
</script>

<script lang="ts">
  import Icon from "@app/components/Icon.svelte";

  interface Props {
    state: State;
  }

  const { state }: Props = $props();
</script>

<style>
  .badge {
    gap: 0.375rem;
    padding-right: 0.625rem;
    font: var(--txt-body-s-regular);
    white-space: nowrap;
  }
  .accepted {
    background-color: var(--color-feedback-success-bg);
    color: var(--color-feedback-success-text);
  }
  .active {
    background-color: var(--color-feedback-warning-bg);
    color: var(--color-feedback-warning-text);
  }
  .rejected {
    background-color: var(--color-feedback-error-bg);
    color: var(--color-feedback-error-text);
  }
</style>

<span
  class="global-chip badge"
  class:accepted={state.status === "accepted"}
  class:active={state.status === "active"}
  class:rejected={state.status === "rejected"}
  title={stateCaption(state)}>
  <Icon name={stateIcon(state)} />
  {labels[state.status]}
</span>
