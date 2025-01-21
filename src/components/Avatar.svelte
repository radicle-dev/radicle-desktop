<script lang="ts">
  import { createIcon } from "@app/lib/blockies";

  const { publicKey }: { publicKey: string } = $props();

  function createContainer(source: string) {
    const seed = source.toLowerCase();
    const avatar = createIcon({
      seed,
      size: 8,
      scale: 16,
    });
    return avatar.toDataURL();
  }
</script>

<style>
  .avatar {
    display: block;
    width: inherit;
    object-fit: cover;
    background-size: cover;
    background-repeat: no-repeat;
    width: 1rem;
    height: 1rem;
    clip-path: var(--1px-corner-fill);
  }
</style>

<img
  title={publicKey}
  src={createContainer(publicKey)}
  class="avatar"
  alt="avatar" />
