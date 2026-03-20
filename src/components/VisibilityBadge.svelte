<script lang="ts">
  import type { Visibility } from "@bindings/repo/Visibility";

  import capitalize from "lodash/capitalize.js";

  import Icon from "@app/components/Icon.svelte";

  interface Props {
    type: Visibility["type"];
  }

  const { type }: Props = $props();
</script>

<style>
  .badge {
    gap: 0.375rem;
    padding-right: 0.625rem;
    font: var(--txt-body-s-regular);
  }

  .public {
    background-color: var(--color-feedback-success-bg);
    color: var(--color-feedback-success-text);
  }
  .private {
    background-color: var(--color-feedback-warning-bg);
    color: var(--color-feedback-warning-text);
  }
</style>

<span
  class="global-chip badge"
  class:public={type === "public"}
  class:private={type === "private"}>
  <Icon name={type === "public" ? "seed" : "lock"} />
  {capitalize(type)}
</span>
