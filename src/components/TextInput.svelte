<script lang="ts">
  import { onMount } from "svelte";

  import Border from "./Border.svelte";

  export let name: string | undefined = undefined;
  export let placeholder: string | undefined = undefined;
  export let value: string | undefined = undefined;

  export let autofocus: boolean = false;
  export let autoselect: boolean = false;
  export let disabled: boolean = false;

  let inputElement: HTMLInputElement | undefined = undefined;
  let focussed = false;

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
</script>

<style>
  input {
    background: var(--color-background-dip);
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

<Border variant={focussed ? "secondary" : "ghost"} styleWidth="100%">
  <input
    on:focus={() => {
      focussed = true;
    }}
    on:blur={() => {
      focussed = false;
    }}
    bind:this={inputElement}
    type="text"
    {name}
    {placeholder}
    {disabled}
    bind:value
    autocomplete="off"
    spellcheck="false"
    on:input
    on:click
    on:change />
</Border>
