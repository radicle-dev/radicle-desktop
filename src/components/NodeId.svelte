<script lang="ts">
  import { truncateId } from "@app/lib/utils";

  import Avatar from "./Avatar.svelte";

  interface Props {
    publicKey: string;
    alias?: string;
    inline?: boolean;
    styleFontSize?: string;
    styleFontWeight?: string;
  }

  const {
    publicKey,
    alias,
    inline = false,
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
  .inline {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
  }
  .inline .alias {
    align-self: baseline;
  }
</style>

<div
  class="avatar-alias"
  class:inline
  style:font-size={styleFontSize}
  style:font-weight={styleFontWeight}>
  <Avatar {publicKey} />
  {#if alias}
    <span class="txt-overflow alias">
      {alias}
    </span>
  {:else}
    <span class="no-alias">
      {truncateId(publicKey)}
    </span>
  {/if}
</div>
