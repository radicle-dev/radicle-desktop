<script lang="ts" module>
  let openActive: (() => void) | undefined;

  // Opens and focuses the search of the list on screen. Returns false when
  // there is none, or it has nothing to search.
  export function openListSearch(): boolean {
    if (openActive === undefined) {
      return false;
    }
    openActive();
    return true;
  }
</script>

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

  function open() {
    show = true;
    // Mounting autofocuses the input, but it may already be open and blurred.
    requestAnimationFrame(() => {
      document
        .querySelector<HTMLInputElement>('input[name="list-search"]')
        ?.focus({ preventScroll: true });
    });
  }

  $effect(() => {
    if (!hasItems) {
      return;
    }
    openActive = open;
    return () => {
      if (openActive === open) {
        openActive = undefined;
      }
    };
  });
</script>

{#if hasItems}
  {#if show}
    <TextInput
      autofocus
      name="list-search"
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
