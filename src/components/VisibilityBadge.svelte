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
  }

  .public {
    background-color: var(--color-fill-diff-green-light);
    color: var(--color-foreground-success);
  }
  .private {
    background-color: var(--color-fill-private);
    color: var(--color-foreground-yellow);
  }
</style>

<span
  class="global-counter badge"
  class:public={type === "public"}
  class:private={type === "private"}>
  <Icon name={type === "public" ? "seedling" : "lock"} />
  {capitalize(type)}
</span>
