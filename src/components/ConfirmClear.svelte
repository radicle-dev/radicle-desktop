<script lang="ts">
  import Button from "@app/components/Button.svelte";
  import Icon from "@app/components/Icon.svelte";

  interface Props {
    clear: () => void;
    count: number;
  }

  const { clear, count }: Props = $props();

  let closed: boolean = $state(true);
</script>

{#if closed}
  <Button
    variant="naked"
    stylePadding="0 0.25rem"
    onclick={() => (closed = false)}>
    <Icon name="clear-all" />
  </Button>
{:else}
  <div class="global-flex txt-body-m-regular">
    <div class="global-flex" style:justify-content="space-between">
      <Button variant="ghost" onclick={clear}>
        <Icon name="clear-all" />
        Clear all {count}
      </Button>
      <Button variant="outline" onclick={() => (closed = true)}>
        <Icon name="close" />Cancel
      </Button>
    </div>
  </div>
{/if}
