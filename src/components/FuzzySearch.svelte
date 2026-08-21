<script lang="ts">
  import type { ComponentProps } from "svelte";

  import { modalStore } from "@app/lib/modal";
  import { isMac, isTyping } from "@app/lib/utils";

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
    styleHeight?: "1.75rem" | "2rem" | "2.5rem";
  }

  /* eslint-disable prefer-const */
  let {
    hasItems = true,
    placeholder,
    icon = "search",
    show = $bindable(),
    value = $bindable(),
    onFocus,
    onSubmit,
    styleHeight = "2rem",
  }: Props = $props();
  /* eslint-enable prefer-const */

  function handleKeydown(event: KeyboardEvent) {
    const auxiliarKey = isMac() ? event.metaKey : event.ctrlKey;
    if (
      auxiliarKey &&
      event.key.toLowerCase() === "f" &&
      hasItems &&
      $modalStore === undefined &&
      !isTyping(event.target)
    ) {
      event.preventDefault();
      show = true;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

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
      {styleHeight}
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
        {styleHeight}
        keyShortcuts="ctrl+f"
        onclick={() => (show = true)}>
        <Icon name={icon} />
      </Button>
    </div>
  {/if}
{/if}
