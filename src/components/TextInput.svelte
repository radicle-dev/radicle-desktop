<script lang="ts">
  import type { Snippet } from "svelte";
  import type { FormEventHandler } from "svelte/elements";

  import { onMount } from "svelte";

  interface Props {
    autofocus?: boolean;
    autoselect?: boolean;
    disabled?: boolean;
    keyShortcuts?: string;
    left?: Snippet;
    name?: string;
    onDismiss?: () => void;
    onFocus?: () => void;
    onBlur?: () => void;
    onSubmit?: () => void;
    oninput?: FormEventHandler<HTMLInputElement>;
    placeholder?: string;
    type?: string;
    valid?: boolean;
    value?: string;
  }

  /* eslint-disable prefer-const */
  let {
    autofocus = false,
    autoselect = false,
    disabled = false,
    keyShortcuts,
    left,
    name,
    onDismiss,
    onFocus,
    onBlur,
    onSubmit,
    oninput,
    placeholder,
    type = "text",
    valid = true,
    value = $bindable(undefined),
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
    background: transparent;
    font: var(--txt-body-m-regular);
    color: var(--color-text-primary);
    line-height: 1.6;
    outline: none;
    text-overflow: ellipsis;
    width: 100%;
    height: 100%;
    margin: 0;
    height: 30px; /* + 2px border = 2rem */
    border: 0;
  }
  input::placeholder {
    color: var(--color-text-secondary);
    opacity: 1 !important;
  }
  input[disabled] {
    cursor: not-allowed;
  }
</style>

<div
  style:border={`1px solid ${!valid ? "var(--color-feedback-error-border)" : focussed ? "var(--color-border-brand)" : "var(--color-border-subtle)"}`}
  style:border-radius="var(--border-radius-sm)"
  style:display="flex"
  style:gap="0.5rem"
  style:align-items="center"
  style:background-color="var(--color-surface-base)"
  style:width="100%">
  {@render left?.()}
  <input
    style:padding={left ? "0.25rem 0.75rem 0.25rem 0" : "0.25rem 0.75rem"}
    aria-keyshortcuts={keyShortcuts}
    onfocus={() => {
      if (onFocus) {
        onFocus();
      }
      focussed = true;
    }}
    onblur={() => {
      if (onBlur) {
        onBlur();
      }
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
</div>
