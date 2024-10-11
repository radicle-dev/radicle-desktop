<script lang="ts">
  import type { Comment } from "@bindings/Comment";
  import type { ComponentProps } from "svelte";

  import { createEventDispatcher } from "svelte";

  import * as utils from "@app/lib/utils";
  import { embed } from "@app/lib/file";

  import Button from "./Button.svelte";
  import Icon from "./Icon.svelte";
  import Markdown from "./Markdown.svelte";
  import Textarea from "./Textarea.svelte";
  import OutlineButton from "./OutlineButton.svelte";

  export let rid: string;
  export let enableAttachments: boolean = false;
  export let placeholder: string = "Leave your comment";
  export let submitCaption: string = "Comment";
  export let focus: boolean = false;
  export let inline: boolean = false;
  export let body: string = "";
  export let embeds: Map<string, Comment["embeds"][0]> = new Map();
  export let submitInProgress: boolean = false;
  export let disallowEmptyBody: boolean = false;
  export let isValid: () => boolean = () => {
    return true;
  };
  export let stylePadding: string | undefined = undefined;
  export let borderVariant: ComponentProps<Textarea>["borderVariant"] = "float";

  let preview: boolean = false;
  let selectionStart = 0;
  let selectionEnd = 0;
  let inputFiles: FileList | undefined = undefined;

  const inputId = `input-label-${crypto.randomUUID()}`;

  const dispatch = createEventDispatcher<{
    submit: { comment: string; embeds: Map<string, Comment["embeds"][0]> };
    close: null;
    click: null;
  }>();

  function submit() {
    dispatch("submit", { comment: body, embeds });
    preview = false;
  }

  const MAX_BLOB_SIZE = 4_194_304;

  function handleFileDrop(event: { detail: DragEvent }) {
    if (!enableAttachments) {
      return;
    }

    event.detail.preventDefault();
    if (event.detail.dataTransfer) {
      attachEmbeds(event.detail.dataTransfer.files);
    }
  }

  function handleFilePaste(event: ClipboardEvent) {
    // Always allow pasting text content.
    if (event.clipboardData && event.clipboardData.files.length === 0) {
      return;
    }

    if (!enableAttachments) {
      return;
    }

    event.preventDefault();
    if (event.clipboardData) {
      attachEmbeds(event.clipboardData.files);
    }
  }

  function handleFileSelect(event: Event) {
    if (!enableAttachments) {
      return;
    }

    event.preventDefault();
    if (inputFiles) {
      attachEmbeds(inputFiles);
    }
  }

  function attachEmbeds(files: FileList) {
    const embedPromise = Array.from(files).map(embed);
    void Promise.all(embedPromise).then(newEmbeds =>
      newEmbeds.forEach(({ oid, name, content }) => {
        if (content.length > MAX_BLOB_SIZE) {
          console.error("File too large: ", embed.name);
          return;
        }
        embeds.set(oid, { name, content });
        const embedText = `![${name}](${oid})\n`;
        body = body
          .slice(0, selectionStart)
          .concat(embedText, body.slice(selectionEnd));
        selectionStart += embedText.length;
        selectionEnd = selectionStart;
      }),
    );
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
    min-height: 6.8rem;
    padding: 0.75rem;
    margin-left: 1px;
    margin-top: 1px;
  }
  label {
    color: var(--color-foreground-contrast);
  }
  label:hover {
    color: var(--color-fill-secondary);
  }
</style>

<div
  class="comment-section"
  aria-label="extended-textarea"
  title=""
  class:inline>
  {#if preview}
    <div class="preview">
      <!-- TODO: pass down embeds -->
      <Markdown {rid} breaks content={body} />
    </div>
  {:else}
    <input
      multiple
      bind:files={inputFiles}
      style:display="none"
      type="file"
      id={inputId}
      on:change={handleFileSelect} />
    <Textarea
      {borderVariant}
      {stylePadding}
      on:drop={handleFileDrop}
      on:paste={handleFilePaste}
      bind:selectionEnd
      bind:selectionStart
      {focus}
      on:submit={submit}
      bind:value={body}
      {placeholder} />
  {/if}
  <div class="actions">
    <OutlineButton
      disabled={submitInProgress}
      variant="ghost"
      onclick={() => {
        preview = false;
        dispatch("close");
      }}>
      Discard
    </OutlineButton>
    {#if !preview}
      <div class="caption">
        {#if enableAttachments}
          Add files by dragging & dropping, <label
            for={inputId}
            style:cursor="pointer">
            selecting
          </label>
          or pasting them.
        {/if}
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
        onclick={submit}>
        {#if submitInProgress}
          Loading...
        {:else}
          {submitCaption}
        {/if}
      </Button>
    </div>
  </div>
</div>
