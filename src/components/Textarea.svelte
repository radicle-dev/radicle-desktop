<script lang="ts">
  import type { FormEventHandler } from "svelte/elements";
  import type { ComponentProps } from "svelte";

  import { tick } from "svelte";

  import * as utils from "@app/lib/utils";

  import Border from "./Border.svelte";

  interface Props {
    borderVariant?: ComponentProps<typeof Border>["variant"];
    focus?: boolean;
    oninput?: FormEventHandler<HTMLTextAreaElement>;
    onkeypress?: FormEventHandler<HTMLTextAreaElement>;
    placeholder?: string;
    selectionEnd?: number;
    selectionStart?: number;
    size?: "grow" | "resizable" | "fixed-height";
    styleMinHeight?: string;
    stylePadding?: string;
    submit: () => Promise<void>;
    value?: string;
  }

  /* eslint-disable prefer-const */
  let {
    borderVariant = "float",
    focus = false,
    oninput,
    onkeypress,
    placeholder = undefined,
    // Defaulting selectionStart and selectionEnd to 0, since no full support yet.
    selectionEnd = $bindable(0),
    selectionStart = $bindable(0),
    size = "grow",
    styleMinHeight = undefined,
    stylePadding = "0.75rem",
    submit,
    value = $bindable(undefined),
  }: Props = $props();
  /* eslint-enable prefer-const */

  let textareaElement: HTMLTextAreaElement | undefined = $state(undefined);
  let focussed = $state(false);

  // We either auto-grow the textarea, or allow the user to resize it. These
  // options are mutually exclusive because a user resized textarea would
  // automatically shrink upon text input otherwise.
  $effect(() => {
    if (textareaElement && size === "grow") {
      // React to changes to the textarea content.
      // eslint-disable-next-line @typescript-eslint/no-unused-expressions
      value;

      // Reset height to 0px on every value change so that the textarea
      // immediately shrinks when all text is deleted.
      textareaElement.style.height = `0px`;
      textareaElement.style.height = `${textareaElement.scrollHeight}px`;
    }
    if (textareaElement && focus) {
      textareaElement.focus();
      focussed = false;
    }
  });

  $effect.pre(() => {
    if (textareaElement) {
      ({ selectionStart, selectionEnd } = textareaElement);
    }
  });

  $effect(() => {
    void tick().then(() => {
      if (textareaElement && focus) {
        textareaElement.setSelectionRange(selectionStart, selectionEnd);
        textareaElement.focus();
      }
    });
  });

  function handleKeydown(event: KeyboardEvent) {
    event.stopPropagation();
    const auxiliarKey = utils.isMac() ? event.metaKey : event.ctrlKey;
    if (auxiliarKey && event.key === "Enter") {
      void submit();
    }
    if (event.key === "Escape") {
      textareaElement?.blur();
    }
  }
</script>

<style>
  textarea {
    background-color: transparent;
    border: 0;
    color: var(--color-foreground-default);
    font-family: inherit;
    height: 5rem;
    width: 100%;
    min-height: 6.375rem;
    resize: none;
    overflow: hidden;
    outline: none;
    line-height: 1rem;
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
  variant={focussed ? "secondary" : borderVariant}
  styleWidth="100%"
  {styleMinHeight}>
  <textarea
    style:min-height={styleMinHeight}
    style:padding={stylePadding}
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
    {oninput}
    {onkeypress}
    onfocus={() => (focussed = true)}
    onblur={() => (focussed = false)}
    onkeydown={handleKeydown}>
  </textarea>
</Border>
