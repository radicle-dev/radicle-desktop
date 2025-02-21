<script lang="ts">
  import type { Author } from "@bindings/cob/Author";
  import type { Comment } from "@bindings/cob/thread/Comment";
  import type { Embed } from "@bindings/cob/thread/Embed";

  import { tick } from "svelte";
  import partial from "lodash/partial";

  import { scrollIntoView } from "@app/lib/utils";

  import Border from "./Border.svelte";
  import CommentComponent from "@app/components/Comment.svelte";
  import ExtendedTextarea from "./ExtendedTextarea.svelte";
  import Icon from "@app/components/Icon.svelte";

  interface Props {
    thread: {
      root: Comment;
      replies: Comment[];
    };
    rid: string;
    canEditComment: (author: string) => true | undefined;
    editComment?: (
      commentId: string,
      body: string,
      embeds: Embed[],
    ) => Promise<void>;
    createReply?: (
      comment: string,
      embeds: Embed[],
      commentId: string,
    ) => Promise<void>;
    reactOnComment?: (
      commentId: string,
      authors: Author[],
      reaction: string,
    ) => Promise<void>;
  }

  const {
    thread,
    rid,
    canEditComment,
    editComment,
    createReply,
    reactOnComment,
  }: Props = $props();

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

  let showReplyForm = $state(false);
  let submitInProgress = $state(false);

  const root = $derived(thread.root);
  const replies = $derived(thread.replies);
  const style = $derived(
    replies.length > 0 || showReplyForm
      ? "--local-clip-path: var(--2px-top-corner-fill)"
      : "--local-clip-path: var(--2px-corner-fill)",
  );
</script>

<style>
  .comments {
    display: flex;
    flex-direction: column;
    width: 100%;
  }

  .top-level-comment {
    position: relative;
    z-index: 1;
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
      {#snippet actions()}
        <Icon name="reply" onclick={toggleReply} />
      {/snippet}
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
              timestamp={reply.edits[0].timestamp}
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
            <ExtendedTextarea
              disallowEmptyBody
              {submitInProgress}
              {rid}
              placeholder="Reply to comment"
              submitCaption="Reply"
              focus
              close={() => (showReplyForm = false)}
              submit={async ({ comment, embeds }) => {
                try {
                  submitInProgress = true;
                  await createReply(
                    comment,
                    Array.from(embeds.values()),
                    root.id,
                  );
                } finally {
                  showReplyForm = false;
                  submitInProgress = false;
                }
              }} />
          </div>
        {/if}
      </div>
    </Border>
  {/if}
</div>
