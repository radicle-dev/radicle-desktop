<script lang="ts">
  import type { ComponentProps } from "svelte";
  import type { Embed } from "@bindings/cob/thread/Embed";
  import * as utils from "@app/lib/utils";
  import { invoke } from "@app/lib/invoke";

  import Button from "./Button.svelte";
  import Icon from "./Icon.svelte";
  import Markdown from "./Markdown.svelte";
  import Textarea from "./Textarea.svelte";
  import OutlineButton from "./OutlineButton.svelte";

  interface Props {
    rid: string;
    placeholder?: string;
    submitCaption?: string;
    focus?: boolean;
    inline?: boolean;
    body?: string;
    embeds?: Map<string, Embed>;
    submitInProgress?: boolean;
    disallowEmptyBody?: boolean;
    isValid?: () => boolean;
    stylePadding?: string;
    borderVariant?: ComponentProps<typeof Textarea>["borderVariant"];
    submit: (opts: {
      comment: string;
      embeds: Map<string, Embed>;
    }) => Promise<void>;
    close: () => void;
  }

  /* eslint-disable prefer-const */
  let {
    rid,
    placeholder = "Leave your comment",
    submitCaption = "Comment",
    focus = false,
    inline = false,
    body = $bindable(""),
    embeds = new Map(),
    submitInProgress = false,
    disallowEmptyBody = false,
    isValid = () => true,
    stylePadding,
    borderVariant = "float",
    submit,
    close,
  }: Props = $props();
  /* eslint-enable prefer-const */

  let preview: boolean = $state(false);
  let selectionStart = $state(0);
  let selectionEnd = $state(0);
  let inputFiles: FileList | undefined = $state(undefined);

  const inputId = `input-label-${crypto.randomUUID()}`;

  async function attachEmbeds(files: File[]) {
    const CHUNK_SIZE = 256 * 1024; // 256 KB
    const uploadLabels = files.map(f => `[Uploading ${f.name}...]()`);
    const oldBody = body;
    body = body
      .slice(0, selectionStart)
      .concat(uploadLabels.join("\n"), body.slice(selectionEnd));

    return Promise.all(
      files.map(async file => {
        const buffer = await file.arrayBuffer();
        const bytes = new Uint8Array(buffer);
        const totalChunks = Math.ceil(bytes.byteLength / CHUNK_SIZE);

        for (let i = 0; i < totalChunks; i++) {
          const chunk = buffer.slice(i * CHUNK_SIZE, (i + 1) * CHUNK_SIZE);
          await invoke("process_chunk", { chunk });
        }

        const oid = await invoke<string>("save_embed", {
          rid,
          name: file.name,
        });
        return `![${file.name}](${oid})`;
      }),
    ).then(texts => {
      const embedsText = texts.join("\n");
      body = oldBody
        .slice(0, selectionStart)
        .concat(embedsText, oldBody.slice(selectionEnd));
      selectionStart += embedsText.length;
      selectionEnd = selectionStart;
    });
  }

  async function handleDrop(e: DragEvent) {
    e.preventDefault();
    if (e.dataTransfer) {
      await attachEmbeds(Array.from(e.dataTransfer.files));
    }
  }

  async function handlePaste(e: ClipboardEvent) {
    e.preventDefault();
    if (e.clipboardData) {
      await attachEmbeds(Array.from(e.clipboardData.files));
    }
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
    gap: 1rem;
  }
  .caption {
    font-size: var(--font-size-small);
    color: var(--color-fill-gray);
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }
  .preview {
    font-size: var(--font-size-small);
    min-height: 109px;
    padding: 0.75rem;
    margin-left: 1px;
    margin-top: 1px;
  }
</style>

<div class="comment-section" aria-label="extended-textarea" class:inline>
  {#if preview}
    <div class="preview">
      <Markdown {rid} breaks content={body} />
    </div>
  {:else}
    <input
      multiple
      bind:files={inputFiles}
      style:display="none"
      type="file"
      id={inputId} />
    <Textarea
      {borderVariant}
      {stylePadding}
      bind:selectionEnd
      bind:selectionStart
      ondrop={handleDrop}
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
      <Icon name="cross" />Discard
    </OutlineButton>
    {#if !preview}
      <div class="caption">
        <Icon name="markdown" />
        Markdown is supported. Press {utils.modifierKey()}↵ to submit.
      </div>
    {/if}
    <div class="buttons">
      <OutlineButton
        variant="ghost"
        disabled={body.trim() === ""}
        onclick={() => (preview = !preview)}>
        <Icon name={preview ? "pen" : "eye"} />{preview ? "Edit" : "Preview"}
      </OutlineButton>
      <Button
        variant="ghost"
        disabled={!isValid() ||
          submitInProgress ||
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
