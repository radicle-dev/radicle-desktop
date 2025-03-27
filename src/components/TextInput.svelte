<script lang="ts">
  import { onMount } from "svelte";

  import Border from "./Border.svelte";
  import type { FormEventHandler } from "svelte/elements";

  interface Props {
    name?: string;
    placeholder?: string;
    value?: string;
    type?: string;
    autofocus?: boolean;
    autoselect?: boolean;
    disabled?: boolean;
    onSubmit?: () => void;
    onDismiss?: () => void;
    valid?: boolean;
    oninput?: FormEventHandler<HTMLInputElement>;
    keyShortcuts?: string;
  }

  /* eslint-disable prefer-const */
  let {
    name,
    placeholder,
    value = $bindable(undefined),
    type = "text",
    autofocus = false,
    autoselect = false,
    disabled = false,
    onSubmit,
    onDismiss,
    valid = true,
    oninput,
    keyShortcuts,
  }: Props = $props();
  /* eslint-enable prefer-const */

  let inputElement: HTMLInputElement | undefined = $state(undefined);
  let focussed = $state(false);

  onMount(() => {
    if (inputElement === undefined) {
      return;
    }
    if (autofocus) {
      // We set preventScroll to true for Svelte animations to work.
      inputElement.focus({ preventScroll: true });
    }
    if (autoselect) {
      inputElement.select();
    }
  });

  function handleKeydown(event: KeyboardEvent) {
    event.stopPropagation();
    if (event.key === "Enter" && valid && onSubmit) {
      onSubmit();
    }

    if (event.key === "Escape" && onDismiss) {
      inputElement?.blur();
      onDismiss();
    }
  }
</script>

<style>
  input {
    background: var(--color-background-ghost);
    font-family: inherit;
    font-size: var(--font-size-small);
    color: var(--color-foreground-contrast);
    line-height: 1.6;
    outline: none;
    text-overflow: ellipsis;
    width: 100%;
    height: 100%;
    margin: 0;
    height: 32px;
    padding: 0.25rem 0.75rem;
    border: 0;
  }
  input::placeholder {
    font-family: var(--font-family-sans-serif);
    color: var(--color-foreground-dim);
    opacity: 1 !important;
  }
  input[disabled] {
    cursor: not-allowed;
  }
</style>

<Border
  variant={valid ? (focussed ? "secondary" : "ghost") : "danger"}
  styleWidth="100%">
  <input
    aria-keyshortcuts={keyShortcuts}
    onfocus={() => {
      focussed = true;
    }}
    onblur={() => {
      focussed = false;
    }}
    bind:this={inputElement}
    {type}
    {name}
    {placeholder}
    {disabled}
    bind:value
    autocomplete="off"
    spellcheck="false"
    onkeydown={handleKeydown}
    {oninput} />
</Border>
