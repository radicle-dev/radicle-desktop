<script lang="ts">
  import { truncateId } from "@app/lib/utils";

  import UserAvatar from "@app/components/UserAvatar.svelte";

  interface Props {
    publicKey: string;
    alias?: string;
    inline?: boolean;
    styleFont?: string;
  }

  const {
    publicKey,
    alias,
    inline = false,
    styleFont = undefined,
  }: Props = $props();
</script>

<style>
  .avatar-alias {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    font: var(--txt-body-m-regular);
  }
  .avatar-container {
    width: 1rem;
    height: 1rem;
    overflow: hidden;
    flex-shrink: 0;
  }
  .avatar-container :global(img) {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .no-alias {
    color: var(--color-text-secondary);
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

<div class="avatar-alias" class:inline style:font={styleFont}>
  <div class="avatar-container">
    <UserAvatar nodeId={publicKey} styleWidth="1rem" />
  </div>
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
