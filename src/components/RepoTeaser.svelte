<script lang="ts">
  import Icon from "@app/components/Icon.svelte";

  interface Props {
    name: string;
    seeding: number;
  }

  const { name, seeding }: Props = $props();
</script>

<style>
  .teaser {
    align-items: center;
    width: 100%;
  }

  .seeding {
    margin-left: auto;
    padding: 0 0.375rem;
    background-color: var(--color-fill-ghost);
    gap: 0.25rem;
  }
</style>

<div class="global-flex teaser">
  <Icon name="repo" />
  <span title={name} class="txt-small txt-overflow txt-semibold">
    {name}
  </span>
  <div class="global-counter txt-small seeding">
    <Icon name="seedling-filled" />
    {seeding}
  </div>
</div>
