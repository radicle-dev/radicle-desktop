<script lang="ts">
  import { afterUpdate, beforeUpdate } from "svelte";

  import Border from "./Border.svelte";

  export let value: string | undefined = undefined;
  export let placeholder: string | undefined = undefined;
  export let focus: boolean = false;
  export let size: "grow" | "resizable" | "fixed-height" = "grow";
  export let styleMinHeight: string | undefined = undefined;

  // Defaulting selectionStart and selectionEnd to 0, since no full support yet.
  export let selectionStart: number = 0;
  export let selectionEnd: number = 0;

  let textareaElement: HTMLTextAreaElement | undefined = undefined;
  let focussed = false;

  // We either auto-grow the textarea, or allow the user to resize it. These
  // options are mutually exclusive because a user resized textarea would
  // automatically shrink upon text input otherwise.
  $: if (textareaElement && size === "grow") {
    // React to changes to the textarea content.
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    value;

    // Reset height to 0px on every value change so that the textarea
    // immediately shrinks when all text is deleted.
    textareaElement.style.height = `0px`;
    textareaElement.style.height = `${textareaElement.scrollHeight}px`;
  }

  $: if (textareaElement && focus) {
    textareaElement.focus();
    focus = false;
  }

  beforeUpdate(() => {
    if (textareaElement) {
      ({ selectionStart, selectionEnd } = textareaElement);
    }
  });

  afterUpdate(() => {
    if (textareaElement && focus) {
      textareaElement.setSelectionRange(selectionStart, selectionEnd);
      textareaElement.focus();
    }
  });
</script>

<style>
  textarea {
    background-color: var(--color-background-dip);
    border: 0;
    color: var(--color-foreground-default);
    font-family: inherit;
    height: 5rem;
    padding: 0.75rem;
    width: 100%;
    min-height: 6.375rem;
    resize: none;
    overflow: hidden;
    outline: none;
  }

  textarea::-webkit-scrollbar-corner {
    background-color: transparent;
  }

  textarea::-webkit-resizer {
    background: url(data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAA4AAAAOCAMAAAAolt3jAAAAAXNSR0IB2cksfwAAAAlwSFlzAAAWJQAAFiUBSVIk8AAAAD9QTFRFAAAAZWZmZmZmZmVmZWVmwsLBwsLCZ2ZmwsPCZmdlZWZnwcLBZmZkYGJjw8LDwsPBZmZnZWZkZ2ZkwMDBWFtcNbXb2AAAABV0Uk5TAP///////////////////////1H/YDRrSAAAAFBJREFUeJxVjUESgCAMA2mqAoqK6P/f6kzjIXIos5NumpI8g5LbpJnNQvDl52mWUYTquqnXwstshpHaTi+o+hHXccoKmHVW9yvIxv218ntivmOYAWpLfqaRAAAAAElFTkSuQmCC);
    background-size: 7px;
    background-repeat: no-repeat;
    background-position: bottom 1px right 1px;
  }

  textarea::placeholder {
    color: var(--color-foreground-dim);
  }
</style>

<Border
  variant={focussed ? "secondary" : "ghost"}
  styleWidth="100%"
  {styleMinHeight}>
  <textarea
    style:min-height={styleMinHeight}
    tabindex="0"
    bind:this={textareaElement}
    bind:value
    aria-label="textarea-comment"
    class="txt-small"
    style:resize={size === "resizable" ? "vertical" : undefined}
    style:overflow={size === "resizable" || size === "fixed-height"
      ? "scroll"
      : undefined}
    {placeholder}
    on:change
    on:click
    on:input
    on:focus={() => (focussed = true)}
    on:blur={() => (focussed = false)}
    on:paste
    on:keypress>
  </textarea>
</Border>
