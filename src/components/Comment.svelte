<script lang="ts">
  import type { Author } from "@bindings/Author";
  import type { Comment } from "@bindings/Comment";
  import type { Edit } from "@bindings/Edit";
  import type { Reaction } from "@bindings/Reaction";

  type Embed = Comment["embeds"][0];

  import { tick } from "svelte";

  import { closeFocused } from "./Popover.svelte";
  import * as utils from "@app/lib/utils";

  import ExtendedTextarea from "@app/components/ExtendedTextarea.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import Markdown from "@app/components/Markdown.svelte";
  import NodeId from "@app/components/NodeId.svelte";
  import ReactionSelector from "@app/components/ReactionSelector.svelte";
  import Reactions from "@app/components/Reactions.svelte";

  export let id: string | undefined = undefined;
  export let rid: string;
  export let author: Author;
  export let body: string;
  export let enableAttachments: boolean = false;
  export let reactions: Reaction[] | undefined = undefined;
  export let embeds: Map<string, Embed> | undefined = undefined;
  export let caption = "commented";
  export let timestamp: number;
  export let lastEdit: Edit | undefined = undefined;
  export let disallowEmptyBody: boolean = false;

  export let editComment:
    | ((body: string, embeds: Comment["embeds"]) => Promise<void>)
    | undefined = undefined;
  export let reactOnComment:
    | ((authors: Author[], reaction: string) => Promise<void>)
    | undefined = undefined;

  let state: "read" | "edit" | "submit" = "read";
</script>

<style>
  .card {
    display: flex;
    flex-direction: column;
    padding: 0.5rem 0;
    gap: 0.5rem;
  }
  .card-header {
    display: flex;
    align-items: center;
    white-space: nowrap;
    flex-wrap: wrap;
    padding: 0 0.75rem;
    min-height: 1.5rem;
    gap: 0.5rem;
    font-size: var(--font-size-small);
  }
  .card-metadata {
    color: var(--color-fill-gray);
    font-size: var(--font-size-small);
  }
  .header-right {
    display: flex;
    margin-left: auto;
    gap: 0.5rem;
  }
  .card-body {
    display: flex;
    align-items: center;
    min-height: 1.625rem;
    word-wrap: break-word;
    font-size: var(--font-size-small);
    padding: 0 1rem;
  }
  .actions {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 0.5rem;
    margin-left: 1rem;
  }
  .timestamp {
    font-size: var(--font-size-small);
    color: var(--color-fill-gray);
  }
  .edit-buttons {
    display: flex;
    gap: 0.25rem;
  }
</style>

<div class="card" {id}>
  <div style:position="relative">
    <div class="card-header">
      <slot class="icon" name="icon" />
      <NodeId {...utils.authorForNodeId(author)} />
      <slot name="caption">{caption}</slot>
      {#if id}
        <Id {id} variant="oid" />
      {/if}
      <span class="timestamp" title={utils.absoluteTimestamp(timestamp)}>
        {utils.formatTimestamp(timestamp)}
      </span>
      {#if lastEdit}
        <div
          class="card-metadata"
          title={utils.formatEditedCaption(
            lastEdit.author,
            lastEdit.timestamp,
          )}>
          • edited
        </div>
      {/if}
      <div class="header-right">
        {#if id && editComment}
          <div class="edit-buttons">
            <Icon
              styleCursor="pointer"
              name="pen"
              onclick={() =>
                state === "edit" ? (state = "read") : (state = "edit")} />
          </div>
        {/if}
        {#if id && reactions && reactOnComment}
          <ReactionSelector
            popoverPositionRight="0"
            popoverPositionBottom="1.5rem"
            {reactions}
            on:select={async ({ detail: { authors, emoji } }) => {
              try {
                await reactOnComment(authors, emoji);
              } finally {
                closeFocused();
              }
            }} />
        {/if}
        <slot name="actions" />
      </div>
    </div>
  </div>

  {#if body.trim() === "" && state === "read"}
    <div class="card-body">
      <span class="txt-missing txt-small" style:line-height="1.625rem">
        No description.
      </span>
    </div>
  {:else}
    <div class="card-body">
      {#if editComment && state !== "read"}
        <ExtendedTextarea
          {body}
          {rid}
          {embeds}
          {enableAttachments}
          {disallowEmptyBody}
          borderVariant="ghost"
          submitInProgress={state === "submit"}
          submitCaption="Save"
          placeholder="Leave a comment"
          on:submit={async ({ detail: { comment, embeds } }) => {
            state = "submit";
            try {
              await editComment(comment, Array.from(embeds.values()));
            } finally {
              state = "read";
            }
          }}
          on:close={async () => {
            body = body;
            await tick();
            state = "read";
          }} />
      {:else}
        <div style:width="100%">
          <div style:overflow="hidden">
            <Markdown {rid} breaks content={body} />
          </div>
        </div>
      {/if}
    </div>
  {/if}
  {#if reactions && reactions.length > 0}
    <div class="actions">
      {#if id && reactions && reactOnComment}
        <ReactionSelector
          popoverPositionLeft="0"
          popoverPositionBottom="1.5rem"
          {reactions}
          on:select={async ({ detail: { authors, emoji } }) => {
            try {
              await reactOnComment(authors, emoji);
            } finally {
              closeFocused();
            }
          }} />
      {/if}
      <Reactions handleReaction={reactOnComment} {reactions} />
    </div>
  {/if}
</div>
