<script lang="ts">
  import { truncateId } from "@app/lib/utils";

  import Avatar from "./Avatar.svelte";

  interface Props {
    publicKey: string;
    alias?: string;
    styleFontSize?: string;
    styleFontWeight?: string;
  }

  const {
    publicKey,
    alias,
    styleFontSize = "var(--font-size-small)",
    styleFontWeight = "var(--font-weight-semibold)",
  }: Props = $props();
</script>

<style>
  .avatar-alias {
    display: flex;
    align-items: center;
    gap: 0.375rem;
  }
  .no-alias {
    color: var(--color-foreground-dim);
  }
</style>

<div
  class="avatar-alias"
  style:font-size={styleFontSize}
  style:font-weight={styleFontWeight}>
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
