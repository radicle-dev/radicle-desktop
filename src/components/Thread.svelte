<script lang="ts">
  import type { Author } from "@bindings/cob/Author";
  import type { Comment } from "@bindings/cob/thread/Comment";
  import type { Embed } from "@bindings/cob/thread/Embed";

  import { tick } from "svelte";
  import partial from "lodash/partial";

  import { scrollIntoView } from "@app/lib/utils";

  import CommentComponent from "@app/components/Comment.svelte";
  import CommentToggleInput from "@app/components/CommentToggleInput.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Border from "./Border.svelte";

  export let thread: {
    root: Comment;
    replies: Comment[];
  };
  export let rid: string;
  export let canEditComment: (author: string) => true | undefined;
  export let editComment:
    | ((commentId: string, body: string, embeds: Embed[]) => Promise<void>)
    | undefined;
  export let createReply:
    | ((commentId: string, comment: string, embeds: Embed[]) => Promise<void>)
    | undefined;
  export let reactOnComment:
    | ((
        commentId: string,
        authors: Author[],
        reaction: string,
      ) => Promise<void>)
    | undefined;

  async function toggleReply() {
    showReplyForm = !showReplyForm;
    if (!showReplyForm) {
      return;
    }

    await tick();
    scrollIntoView(`reply-${root.id}`, {
      behavior: "smooth",
      block: "center",
    });
  }

  let showReplyForm = false;

  $: root = thread.root;
  $: replies = thread.replies;

  $: style =
    replies.length > 0
      ? "--local-clip-path: var(--2px-top-corner-fill)"
      : "--local-clip-path: var(--2px-corner-fill)";
</script>

<style>
  .comments {
    display: flex;
    flex-direction: column;
    width: 100%;
  }

  .top-level-comment {
    position: relative;
  }
  /* We put the background and clip-path in a separate element to prevent
     popovers being clipped in the main element. */
  .top-level-comment::after {
    position: absolute;
    z-index: -1;
    content: " ";
    background-color: var(--color-background-float);
    clip-path: var(--local-clip-path);
    width: 100%;
    height: 100%;
    top: 0;
  }
</style>

<div class="comments" {style}>
  <div class="top-level-comment">
    <CommentComponent
      disallowEmptyBody
      {rid}
      id={root.id}
      lastEdit={root.edits.length > 1 ? root.edits.at(-1) : undefined}
      author={root.author}
      reactions={root.reactions}
      timestamp={root.edits.slice(-1)[0].timestamp}
      body={root.edits.slice(-1)[0].body}
      editComment={editComment &&
        canEditComment(root.author.did) &&
        partial(editComment, root.id)}
      reactOnComment={reactOnComment && partial(reactOnComment, root.id)}>
      <svelte:fragment slot="actions">
        <Icon name="reply" onclick={toggleReply} styleCursor="pointer" />
      </svelte:fragment>
    </CommentComponent>
  </div>
  {#if replies.length > 0 || (createReply && showReplyForm)}
    <Border variant="float" flatTop>
      <div style:width="100%">
        {#if replies.length > 0}
          {#each replies as reply}
            <CommentComponent
              disallowEmptyBody
              {rid}
              lastEdit={reply.edits.length > 1 ? reply.edits.at(-1) : undefined}
              id={reply.id}
              author={reply.author}
              caption="replied"
              reactions={reply.reactions}
              timestamp={reply.edits.slice(-1)[0].timestamp}
              body={reply.edits.slice(-1)[0].body}
              editComment={editComment &&
                canEditComment(reply.author.did) &&
                partial(editComment, reply.id)}
              reactOnComment={reactOnComment &&
                partial(reactOnComment, reply.id)} />
          {/each}
        {/if}
        {#if createReply && showReplyForm}
          <div id={`reply-${root.id}`} style:padding="1rem">
            <CommentToggleInput
              disallowEmptyBody
              {rid}
              focus
              inline
              onclose={() => (showReplyForm = false)}
              placeholder="Reply to comment"
              submitCaption="Reply"
              onexpand={toggleReply}
              submit={partial(createReply, root.id)} />
          </div>
        {/if}
        <div></div>
      </div>
    </Border>
  {/if}
</div>
