<script lang="ts">
  import type { Author } from "@bindings/cob/Author";
  import type { Edit } from "@bindings/cob/patch/Edit";
  import type { Reaction } from "@bindings/cob/Reaction";
  import type { Embed } from "@bindings/cob/thread/Embed";
  import type { Snippet } from "svelte";

  import { tick } from "svelte";

  import * as utils from "@app/lib/utils";

  import ExtendedTextarea from "@app/components/ExtendedTextarea.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import Markdown from "@app/components/Markdown.svelte";
  import NodeId from "@app/components/NodeId.svelte";
  import { closeFocused } from "@app/components/Popover.svelte";
  import Reactions from "@app/components/Reactions.svelte";
  import ReactionSelector from "@app/components/ReactionSelector.svelte";

  interface Props {
    actions?: Snippet;
    beforeTimestamp?: Snippet;
    id?: string;
    rid: string;
    currentUserNid?: string;
    author: Author;
    body?: string;
    reactions?: Reaction[];
    embeds?: Map<string, Embed>;
    caption?: string;
    timestamp?: number;
    lastEdit?: Edit;
    disallowEmptyBody?: boolean;
    emptyBodyTooltip?: string;
    editComment?: (body: string, embeds: Embed[]) => Promise<void>;
    reactOnComment?: (authors: Author[], reaction: string) => Promise<void>;
    deleteComment?: () => Promise<void>;
    styleWidth?: string;
    // See `ExtendedTextarea`
    disableAttachments?: boolean | string;
  }

  /* eslint-disable prefer-const */
  let {
    actions,
    beforeTimestamp,
    id,
    rid,
    currentUserNid,
    author,
    body = $bindable(),
    reactions,
    embeds,
    caption = "commented",
    timestamp,
    lastEdit,
    disallowEmptyBody = false,
    editComment,
    reactOnComment,
    deleteComment,
    styleWidth,
    emptyBodyTooltip,
    disableAttachments,
  }: Props = $props();
  /* eslint-enable prefer-const */

  let state: "read" | "edit" | "submit" = $state("read");

  async function toggleEdit() {
    if (state === "read") {
      state = "edit";
      await tick();
      utils.scrollIntoView(`edit-${id}`, {
        behavior: "smooth",
        block: "center",
      });
    } else if (state === "edit") {
      state = "read";
    }
  }
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
    font: var(--txt-body-m-regular);
  }
  .card-metadata {
    color: var(--color-text-quaternary);
    font: var(--txt-body-m-regular);
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
    font: var(--txt-body-m-regular);
    padding: 0 0.75rem;
  }
  .actions {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 0.5rem;
    margin-left: 1rem;
  }
  .timestamp {
    font: var(--txt-body-m-regular);
    color: var(--color-text-quaternary);
  }
  .icon-button {
    cursor: pointer;
  }
</style>

<div class="card" {id} style:width={styleWidth}>
  <div style:position="relative">
    <div class="card-header">
      <NodeId {...utils.authorForNodeId(author)} />
      {caption}
      {#if id}
        <Id {id} clipboard={id} />
      {/if}
      {#if beforeTimestamp}
        {@render beforeTimestamp()}
      {/if}
      {#if timestamp}
        <span class="timestamp" title={utils.absoluteTimestamp(timestamp)}>
          {utils.formatTimestamp(timestamp)}
        </span>
      {/if}
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
        {#if editComment}
          <div class="icon-button">
            <Icon name="edit" onclick={toggleEdit} />
          </div>
        {/if}
        {#if deleteComment}
          <div class="icon-button">
            <Icon name="trash" onclick={deleteComment} />
          </div>
        {/if}
        {#if reactions && reactOnComment}
          <ReactionSelector
            placement="top-end"
            {reactions}
            select={async ({ authors, emoji }) => {
              try {
                await reactOnComment(authors, emoji);
              } finally {
                closeFocused();
              }
            }} />
        {/if}
        {@render actions?.()}
      </div>
    </div>
  </div>

  {#if (body === undefined || body?.trim() === "") && state === "read"}
    <div class="card-body">
      <span class="txt-missing txt-body-m-regular" style:line-height="1.625rem">
        No description.
      </span>
    </div>
  {:else}
    <div class="card-body">
      {#if editComment && state !== "read"}
        <div id={`edit-${id}`} style:width="100%">
          <ExtendedTextarea
            focus
            {body}
            {rid}
            {embeds}
            {disallowEmptyBody}
            {emptyBodyTooltip}
            {disableAttachments}
            borderVariant="ghost"
            submitVariant="secondary"
            submitInProgress={state === "submit"}
            submitCaption="Save"
            placeholder="Leave a comment"
            submit={async ({ comment, embeds }) => {
              state = "submit";
              try {
                await editComment(comment, Array.from(embeds.values()));
              } finally {
                state = "read";
              }
            }}
            close={async () => {
              body = body;
              await tick();
              state = "read";
            }} />
        </div>
      {:else}
        <div style:width="100%">
          <div style:overflow="hidden">
            <Markdown {rid} breaks content={body ?? ""} />
          </div>
        </div>
      {/if}
    </div>
  {/if}
  {#if reactions && reactions.length > 0}
    <div class="actions">
      {#if id && reactions && reactOnComment}
        <ReactionSelector
          placement="top-start"
          {reactions}
          select={async ({ authors, emoji }) => {
            try {
              await reactOnComment(authors, emoji);
            } finally {
              closeFocused();
            }
          }} />
      {/if}
      <Reactions handleReaction={reactOnComment} {currentUserNid} {reactions} />
    </div>
  {/if}
</div>
