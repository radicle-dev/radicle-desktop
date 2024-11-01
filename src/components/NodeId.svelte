<script lang="ts">
  import { truncateId } from "@app/lib/utils";

  import Avatar from "./Avatar.svelte";

  interface Props {
    publicKey: string;
    alias?: string;
    styleFontSize?: string;
    styleFontFamily?: string;
  }

  const {
    publicKey,
    alias,
    styleFontSize = "var(--font-size-small)",
    styleFontFamily = "var(--font-family-monospace)",
  }: Props = $props();
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
  <Avatar {publicKey} />
  {#if alias}
    <span class="txt-overflow">
      {alias}
    </span>
  {:else}
    <span class="no-alias">
      {truncateId(publicKey)}
    </span>
  {/if}
</div>
