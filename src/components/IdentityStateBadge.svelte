<script lang="ts" module>
  import type { State } from "@bindings/identity/State";

  import { formatOid } from "@app/lib/utils";

  const icons = {
    active: "hourglass",
    accepted: "checkmark",
    rejected: "close",
    redacted: "trash",
  } as const;

  export function stateIcon(state: State): (typeof icons)[State["status"]] {
    return icons[state.status];
  }

  // Spelled out because "rejected" alone does not say whether the delegates
  // voted it down or a competing revision won.
  export function stateCaption(state: State): string | undefined {
    if (state.status === "rejected") {
      if (state.reason.type === "vote") return "Voted down by the delegates";
      if (state.reason.type === "parent")
        return "Its parent revision was rejected";
      return `Superseded by ${formatOid(state.reason.revision)}`;
    }
    if (state.status === "redacted") {
      if (state.reason.type === "author") return "Withdrawn by its author";
      return "Its parent revision was redacted";
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

  const label = {
    active: "Proposed",
    accepted: "Accepted",
    rejected: "Rejected",
    redacted: "Redacted",
  };
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
  .rejected,
  .redacted {
    background-color: var(--color-feedback-error-bg);
    color: var(--color-feedback-error-text);
  }
</style>

<span
  class="global-chip badge"
  class:accepted={state.status === "accepted"}
  class:active={state.status === "active"}
  class:rejected={state.status === "rejected"}
  class:redacted={state.status === "redacted"}
  title={stateCaption(state)}>
  <Icon name={stateIcon(state)} />
  {label[state.status]}
</span>
