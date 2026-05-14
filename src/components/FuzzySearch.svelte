<script lang="ts">
  import type { ComponentProps } from "svelte";

  import Button from "@app/components/Button.svelte";
  import Icon from "@app/components/Icon.svelte";
  import TextInput from "@app/components/TextInput.svelte";

  interface Props {
    hasItems?: boolean;
    placeholder: string;
    icon?: ComponentProps<typeof Icon>["name"];
    show: boolean;
    value: string;
    onFocus?: () => void;
    onSubmit?: () => void;
  }

  /* eslint-disable prefer-const */
  let {
    hasItems = true,
    placeholder,
    icon = "filter",
    show = $bindable(),
    value = $bindable(),
    onFocus,
    onSubmit,
  }: Props = $props();
  /* eslint-enable prefer-const */
</script>

{#if hasItems}
  {#if show}
    <TextInput
      autofocus
      {onFocus}
      {onSubmit}
      onBlur={() => {
        if (value === "") {
          show = false;
        }
      }}
      onDismiss={() => {
        value = "";
        show = false;
      }}
      {placeholder}
      keyShortcuts="ctrl+f"
      bind:value>
      {#snippet left()}
        <div
          style:color="var(--color-text-secondary)"
          style:padding-left="0.5rem">
          <Icon name={icon} />
        </div>
      {/snippet}
    </TextInput>
  {:else}
    <div style:display="flex" style:padding="0 1px">
      <Button
        variant="naked"
        keyShortcuts="ctrl+f"
        onclick={() => (show = true)}>
        <Icon name="filter" />
      </Button>
    </div>
  {/if}
{/if}
