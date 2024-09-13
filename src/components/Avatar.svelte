<script lang="ts">
  import { createIcon } from "@app/lib/blockies";

  export let nodeId: string;

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

<img title={nodeId} src={createContainer(nodeId)} class="avatar" alt="avatar" />
