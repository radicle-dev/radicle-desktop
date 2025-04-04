<script lang="ts">
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import type { ComponentProps } from "svelte";

  import * as utils from "@app/lib/utils";

  import type { Embed } from "@bindings/cob/thread/Embed";
  import { invoke } from "@app/lib/invoke";
  import { listen } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";

  import Button from "./Button.svelte";
  import Icon from "./Icon.svelte";
  import Markdown from "./Markdown.svelte";
  import Textarea from "./Textarea.svelte";
  import OutlineButton from "./OutlineButton.svelte";

  interface Props {
    textAreaSize?: ComponentProps<typeof Textarea>["size"];
    styleMinHeight?: string;
    rid: string;
    placeholder?: string;
    submitCaption?: string;
    focus?: boolean;
    inline?: boolean;
    body?: string;
    embeds?: Map<string, Embed>;
    submitInProgress?: boolean;
    disableSubmit?: boolean;
    disallowEmptyBody?: boolean;
    emptyBodyTooltip?: string;
    isValid?: () => boolean;
    preview?: boolean;
    stylePadding?: string;
    borderVariant?: ComponentProps<typeof Textarea>["borderVariant"];
    submit: (opts: {
      comment: string;
      embeds: Map<string, Embed>;
    }) => Promise<void>;
    close: () => void;
    allowAttachments?: boolean;
  }

  /* eslint-disable prefer-const */
  let {
    textAreaSize,
    preview = $bindable(false),
    styleMinHeight,
    rid,
    placeholder = "Leave your comment",
    submitCaption = "Comment",
    focus = false,
    inline = false,
    body = $bindable(""),
    embeds = new Map(),
    submitInProgress = false,
    disableSubmit = false,
    disallowEmptyBody = false,
    emptyBodyTooltip,
    isValid = () => true,
    stylePadding,
    borderVariant = "float",
    submit,
    close,
    allowAttachments = true,
  }: Props = $props();
  /* eslint-enable prefer-const */

  let selectionStart = $state(body.length);
  let selectionEnd = $state(body.length);
  let draggingOver = $state(false);
  let dragEnterUnlistenFn: UnlistenFn | undefined = undefined;
  let dragLeaveUnlistenFn: UnlistenFn | undefined = undefined;
  let dragDropUnlistenFn: UnlistenFn | undefined = undefined;

  function updateBodyAndSelection(input: string[], pre: string, after: string) {
    const allEmbeds = input.join("");
    body = pre.concat(allEmbeds, after);
    selectionStart = pre.length + allEmbeds.length;
    selectionEnd = pre.length + allEmbeds.length;
  }

  function splitBody() {
    return [body.substring(0, selectionStart), body.substring(selectionStart)];
  }

  onMount(async () => {
    if (window.__TAURI_INTERNALS__) {
      if (allowAttachments) {
        dragEnterUnlistenFn = await listen("tauri://drag-enter", () => {
          draggingOver = true;
        });

        dragLeaveUnlistenFn = await listen("tauri://drag-leave", () => {
          draggingOver = false;
        });

        dragDropUnlistenFn = await listen<{
          paths: string[];
          position: { x: number; y: number };
        }>("tauri://drag-drop", async event => {
          draggingOver = false;
          const [preBody, afterBody] = splitBody();

          return Promise.all(
            event.payload.paths.map(async path => {
              const pathSegments = path.split("/");
              const name = pathSegments[pathSegments.length - 1];
              const uploadLabel = `[Uploading ${name}...]()\n`;

              body = preBody.concat(uploadLabel, afterBody);
              const oid = await invoke<string>("save_embed_by_path", {
                rid,
                path,
              });
              embeds.set(oid, { name, content: `git:${oid}` });
              return `[${name}](${oid})\n`;
            }),
          ).then(texts => updateBodyAndSelection(texts, preBody, afterBody));
        });
      }
    }
  });

  onDestroy(() => {
    if (dragEnterUnlistenFn) dragEnterUnlistenFn();
    if (dragLeaveUnlistenFn) dragLeaveUnlistenFn();
    if (dragDropUnlistenFn) dragDropUnlistenFn();
  });

  async function attachEmbedsByPaths(paths: string[]) {
    const [preBody, afterBody] = splitBody();

    return Promise.all(
      paths.map(async path => {
        const pathSegments = path.split("/");
        const name = pathSegments[pathSegments.length - 1];
        const uploadLabel = `[Uploading ${name}...]()\n`;
        body = preBody.concat(uploadLabel, afterBody);
        const oid = await invoke<string>("save_embed_by_path", {
          rid,
          path,
        });
        embeds.set(oid, { name: name ?? path, content: `git:${oid}` });
        return `[${name}](${oid})\n`;
      }),
    ).then(texts => updateBodyAndSelection(texts, preBody, afterBody));
  }

  async function handlePaste(e: ClipboardEvent) {
    if (!allowAttachments) {
      return;
    }

    if (e.clipboardData?.files && e.clipboardData.files.length > 0) {
      e.preventDefault();
      const [preBody, afterBody] = splitBody();
      // We read the buffer on the backend, if it's a image buffer.
      if (e.clipboardData.items.length === 1) {
        const file = e.clipboardData.files[0];
        const uploadLabel = `[Uploading...]()\n`;
        body = preBody.concat(uploadLabel, afterBody);
        const oid = await invoke<string>("save_embed_by_clipboard", {
          name: file.name,
          rid,
        });
        embeds.set(oid, { name: file.name, content: `git:${oid}` });
        body = preBody.concat(`[${file.name}](${oid})\n`, afterBody);
      } else {
        return Promise.all(
          Array.from(e.clipboardData.files).map(async file => {
            const arrayBuffer = await file.arrayBuffer();
            const bytes = new Uint8Array(arrayBuffer);
            const uploadLabel = `[Uploading ${file.name}...]()\n`;
            body = preBody.concat(uploadLabel, afterBody);
            const oid = await invoke<string>("save_embed_by_bytes", {
              rid,
              name: file.name,
              bytes,
            });
            embeds.set(oid, { name: file.name, content: `git:${oid}` });
            return `[${file.name}](${oid})\n`;
          }),
        ).then(texts => updateBodyAndSelection(texts, preBody, afterBody));
      }
    } else {
      // In case that the clipboard data isn't an array of files,
      // we want to make use of the default behavior and insert the clipboard content.
    }
  }

  function selectFiles() {
    void open({ multiple: true }).then(paths => {
      if (paths) {
        void attachEmbedsByPaths(paths);
      }
    });
  }

  function submitFn() {
    void submit({ comment: body, embeds })
      .then(() => (preview = false))
      .catch(e => {
        console.error(e);
      });
  }
</script>

<style>
  .comment-section {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 1rem;
    width: 100%;
    flex: 1;
    font-family: var(--font-family-sans-serif);
  }
  .inline {
    border: 0;
    padding: 0;
  }
  .actions {
    display: flex;
    flex-direction: row;
    align-items: center;
    width: 100%;
    gap: 1rem;
  }
  .buttons {
    display: flex;
    margin-left: auto;
    gap: 0.5rem;
  }

  .preview {
    width: 100%;
    font-size: var(--font-size-small);
    min-height: 109px;
    padding: 0.75rem;
    margin-left: 1px;
    margin-top: 1px;
    flex: 1;
  }
</style>

<div class="comment-section" aria-label="extended-textarea" class:inline>
  {#if preview}
    <div class="preview">
      {#if body.trim().length === 0}
        <span class="txt-missing">Nothing to preview.</span>
      {:else}
        <Markdown {rid} breaks content={body} />
      {/if}
    </div>
  {:else}
    <Textarea
      size={textAreaSize}
      styleAlignItems="flex-start"
      {draggingOver}
      {borderVariant}
      {stylePadding}
      {styleMinHeight}
      bind:selectionEnd
      bind:selectionStart
      onpaste={handlePaste}
      {focus}
      submit={() => submit({ comment: body, embeds })}
      bind:value={body}
      {placeholder} />
  {/if}
  <div class="actions">
    <OutlineButton
      disabled={submitInProgress}
      variant="ghost"
      onclick={() => {
        preview = false;
        close();
      }}>
      <Icon name="cross" />
      <span class="global-hide-on-small-desktop-down">Discard</span>
    </OutlineButton>
    {#if !preview}
      <div
        class="txt-overflow txt-small txt-missing"
        title={`${allowAttachments ? "Drag and drop files to add them. " : ""}Markdown is supported. Press ${utils.modifierKey()}↵ to submit.`}>
        {#if allowAttachments}Drag and drop files to add them.{/if}
        <Icon
          name="markdown"
          styleDisplay="inline"
          styleVerticalAlign="text-top" />
        Markdown is supported. Press {utils.modifierKey()}↵ to submit.
      </div>
    {/if}
    <div class="buttons">
      {#if allowAttachments}
        <OutlineButton variant="ghost" onclick={selectFiles} disabled={preview}>
          <Icon name="attachment" />
          Attach
        </OutlineButton>
      {/if}
      <OutlineButton variant="ghost" onclick={() => (preview = !preview)}>
        <Icon name={preview ? "pen" : "eye"} />
        {preview ? "Edit" : "Preview"}
      </OutlineButton>
      <Button
        variant="ghost"
        title={emptyBodyTooltip}
        disabled={!isValid() ||
          submitInProgress ||
          disableSubmit ||
          (disallowEmptyBody && body.trim() === "")}
        onclick={submitFn}>
        <Icon name="checkmark" />
        {#if submitInProgress}
          Saving…
        {:else}
          {submitCaption}
        {/if}
      </Button>
    </div>
  </div>
</div>
