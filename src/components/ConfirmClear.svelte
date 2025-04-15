<script lang="ts">
  import Button from "./Button.svelte";
  import Icon from "@app/components/Icon.svelte";
  import NakedButton from "@app/components/NakedButton.svelte";
  import OutlineButton from "./OutlineButton.svelte";

  interface Props {
    clear: () => void;
    count: number;
  }

  const { clear, count }: Props = $props();

  let closed: boolean = $state(true);
</script>

{#if closed}
  <NakedButton
    stylePadding="0 0.25rem"
    variant="ghost"
    onclick={() => (closed = false)}>
    <Icon name="broom-double" />
  </NakedButton>
{:else}
  <div class="global-flex txt-small">
    <div class="global-flex" style:justify-content="space-between">
      <Button variant="ghost" onclick={clear}>
        <Icon name="broom-double" />
        Clear all {count}
      </Button>
      <OutlineButton variant="ghost" onclick={() => (closed = true)}>
        <Icon name="cross" />Cancel
      </OutlineButton>
    </div>
  </div>
{/if}
