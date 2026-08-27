<script lang="ts">
  import type {
    ClipboardEventHandler,
    FormEventHandler,
  } from "svelte/elements";

  import { onMount, tick } from "svelte";

  import type { MarkdownFormat, TextEdit } from "@app/lib/markdownFormat";
  import {
    applyMarkdownFormat,
    applyTextEdit,
    pasteLinkEdit,
  } from "@app/lib/markdownFormat";
  import * as utils from "@app/lib/utils";

  interface Props {
    draggingOver?: boolean;
    borderVariant?: "float" | "ghost";
    onpaste?: ClipboardEventHandler<HTMLTextAreaElement>;
    focus?: boolean;
    oninput?: FormEventHandler<HTMLTextAreaElement>;
    onkeypress?: FormEventHandler<HTMLTextAreaElement>;
    placeholder?: string;
    selectionEnd?: number;
    selectionStart?: number;
    size?: "grow" | "resizable" | "fixed-height";
    styleMinHeight?: string;
    styleAlignItems?: string;
    stylePadding?: string;
    submit: () => Promise<void>;
    value?: string;
  }

  /* eslint-disable prefer-const */
  let {
    draggingOver,
    borderVariant = "float",
    focus = false,
    onpaste,
    oninput,
    onkeypress,
    placeholder = undefined,
    // Defaulting selectionStart and selectionEnd to 0, since no full support yet.
    selectionEnd = $bindable(0),
    selectionStart = $bindable(0),
    size = "grow",
    styleAlignItems = undefined,
    styleMinHeight = undefined,
    stylePadding = "0.75rem",
    submit,
    value = $bindable(undefined),
  }: Props = $props();
  /* eslint-enable prefer-const */

  const borderColors: Record<NonNullable<Props["borderVariant"]>, string> = {
    ghost: "var(--color-border-subtle)",
    float: "var(--color-border-subtle)",
  };

  let textareaElement: HTMLTextAreaElement | undefined = $state(undefined);
  let focussed = $state(false);

  onMount(() => {
    if (textareaElement) {
      // The selectionchange event listener doesn't modify the selection on Enter.
      textareaElement.addEventListener("keydown", (event: KeyboardEvent) => {
        if (event.key === "Enter") {
          selectionStart += 1;
          selectionEnd += 1;
        }
      });
      textareaElement.addEventListener("selectionchange", (event: Event) => {
        if (
          event.target &&
          "selectionStart" in event.target &&
          "selectionEnd" in event.target
        ) {
          selectionStart = event.target.selectionStart as number;
          selectionEnd = event.target.selectionEnd as number;
        }
      });
    }
  });

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
  });

  $effect(() => {
    if (textareaElement && focus) {
      textareaElement.focus();
      focus = false;
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

  const formatShortcuts: Record<string, MarkdownFormat> = {
    b: "bold",
    i: "italic",
    e: "code",
    k: "link",
  };

  // Writing through `insertText` keeps the edit on the textarea's native undo
  // stack, so Cmd+Z treats it the same way it treats typing. Assigning the
  // value would clear that stack instead, so it is only a fallback.
  function applyEdit(element: HTMLTextAreaElement, edit: TextEdit) {
    element.setSelectionRange(edit.from, edit.to);
    const applied = edit.text
      ? document.execCommand("insertText", false, edit.text)
      : document.execCommand("delete");
    if (!applied) {
      value = applyTextEdit(element.value, edit);
    }

    selectionStart = edit.selectionStart;
    selectionEnd = edit.selectionEnd;
    void tick().then(() =>
      element.setSelectionRange(edit.selectionStart, edit.selectionEnd),
    );
  }

  // Pasting a link over a selection wraps the selected text in markdown link
  // syntax, matching GitHub and other markdown editors. Anything else falls
  // through to the browser's default paste.
  function handlePaste(
    event: ClipboardEvent & {
      currentTarget: EventTarget & HTMLTextAreaElement;
    },
  ) {
    void onpaste?.(event);
    if (event.defaultPrevented) {
      return;
    }

    const element = event.currentTarget;
    const start = element.selectionStart;
    const end = element.selectionEnd;
    if (start === end) {
      return;
    }

    const url = utils.pastedLinkUrl(
      event.clipboardData?.getData("text/plain") ?? "",
    );
    if (!url) {
      return;
    }

    event.preventDefault();
    applyEdit(element, pasteLinkEdit(element.value, start, end, url));
  }

  function handleKeydown(event: KeyboardEvent) {
    event.stopPropagation();
    const auxiliarKey = utils.isMac() ? event.metaKey : event.ctrlKey;
    if (auxiliarKey && event.key === "Enter") {
      void submit();
    }
    if (event.key === "Escape") {
      textareaElement?.blur();
    }

    // Shift and Alt are left alone so that combinations the platform or the app
    // binds elsewhere still reach it.
    const format =
      auxiliarKey && !event.shiftKey && !event.altKey
        ? formatShortcuts[event.key.toLowerCase()]
        : undefined;
    if (format && textareaElement) {
      event.preventDefault();
      applyEdit(
        textareaElement,
        applyMarkdownFormat(
          format,
          textareaElement.value,
          textareaElement.selectionStart,
          textareaElement.selectionEnd,
        ),
      );
    }
  }
</script>

<style>
  textarea {
    background-color: transparent;
    border: 0;
    color: var(--color-text-secondary);
    font-family: inherit;
    height: 100%;
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
    color: var(--color-text-secondary);
  }

  textarea::-webkit-scrollbar {
    width: 7px;
    height: 7px;
  }

  textarea::-webkit-scrollbar-track {
    background-color: transparent;
  }

  textarea::-webkit-scrollbar-thumb {
    background-color: var(--color-surface-subtle);
    border-radius: var(--border-radius-md);
  }

  .dragover {
    position: absolute;
    opacity: 0.5;
    display: flex;
    justify-content: center;
    align-items: center;
    width: 100%;
    height: 100%;
    background-color: var(--color-surface-canvas);
  }
</style>

<div
  style:border={`1px solid ${focussed ? "var(--color-border-brand)" : borderColors[borderVariant]}`}
  style:border-radius="var(--border-radius-sm)"
  style:display="flex"
  style:gap="0.5rem"
  style:align-items={styleAlignItems}
  style:background-color="var(--color-surface-base)"
  style:position="relative"
  style:width="100%"
  style:min-height={styleMinHeight}>
  <textarea
    style:min-height={styleMinHeight}
    style:padding={stylePadding}
    tabindex="0"
    bind:this={textareaElement}
    bind:value
    aria-label="textarea-comment"
    class="txt-body-m-regular"
    style:resize={size === "resizable" ? "vertical" : undefined}
    style:overflow={size === "resizable" || size === "fixed-height"
      ? "scroll"
      : undefined}
    {placeholder}
    onpaste={handlePaste}
    {oninput}
    {onkeypress}
    onfocus={() => (focussed = true)}
    onblur={() => (focussed = false)}
    onkeydown={handleKeydown}>
  </textarea>
  {#if draggingOver}
    <div class="txt-body-m-regular dragover">
      Drop files to add them as embeds. Embeds are limited to 10MB.
    </div>
  {/if}
</div>
