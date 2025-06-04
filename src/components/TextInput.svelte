<script lang="ts">
  import type { FormEventHandler } from "svelte/elements";
  import type { Snippet } from "svelte";

  import { onMount } from "svelte";

  import Border from "./Border.svelte";

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
    height: 2rem;
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
</Border>
