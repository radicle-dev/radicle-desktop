<script lang="ts">
  import { truncateId } from "@app/lib/utils";

  import Avatar from "./Avatar.svelte";

  export let nodeId: string;
  export let alias: string | undefined = undefined;
  export let styleFontSize: string | undefined = "var(--font-size-small)";
  export let styleFontFamily: string | undefined =
    "var(--font-family-monospace)";
</script>

<style>
  .avatar-alias {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    font-weight: var(--font-weight-semibold);
  }
  .no-alias {
    color: var(--color-foreground-dim);
  }
</style>

<div
  class="avatar-alias"
  style:font-size={styleFontSize}
  style:font-family={styleFontFamily}>
  <Avatar {nodeId} />
  {#if alias}
    <span class="txt-overflow">
      {alias}
    </span>
  {:else}
    <span class="no-alias">
      {truncateId(nodeId)}
    </span>
  {/if}
</div>
