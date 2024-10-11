<script lang="ts">
  import type { ComponentProps } from "svelte";
  import type { Embed } from "@bindings/cob/thread/Embed";

  import { createEventDispatcher } from "svelte";

  import * as utils from "@app/lib/utils";

  import Button from "./Button.svelte";
  import Icon from "./Icon.svelte";
  import Markdown from "./Markdown.svelte";
  import Textarea from "./Textarea.svelte";
  import OutlineButton from "./OutlineButton.svelte";

  export let rid: string;
  export let placeholder: string = "Leave your comment";
  export let submitCaption: string = "Comment";
  export let focus: boolean = false;
  export let inline: boolean = false;
  export let body: string = "";
  export let embeds: Map<string, Embed> = new Map();
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
    submit: {
      comment: string;
      embeds: Map<string, Embed>;
    };
    close: null;
  }>();

  function submit() {
    dispatch("submit", { comment: body, embeds });
    preview = false;
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
