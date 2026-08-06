<script lang="ts">
  import type { CommentOrigin } from "@app/components/Comment.svelte";
  import type { Author } from "@bindings/cob/Author";
  import type { CodeLocation } from "@bindings/cob/thread/CodeLocation";
  import type { Embed } from "@bindings/cob/thread/Embed";
  import type { Thread } from "@bindings/cob/thread/Thread";

  import { tick } from "svelte";

  import { scrollIntoView } from "@app/lib/utils";

  import CommentComponent from "@app/components/Comment.svelte";
  import ExtendedTextarea from "@app/components/ExtendedTextarea.svelte";
  import Icon from "@app/components/Icon.svelte";

  interface Props {
    thread: Thread<CodeLocation>;
    rid: string;
    currentUserNid?: string;
    // Whether the current user may edit or delete a comment by this author.
    // The protocol reserves both for the comment's own author.
    canModifyComment: (author: string) => true | undefined;
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
    deleteComment?: (commentId: string) => Promise<void>;
    changeCommentStatus?: (
      commentId: string,
      resolved: boolean,
    ) => Promise<void>;
    canResolve?: boolean;
    // Set for threads rendered inside a diff, where there is no surrounding
    // activity timeline for the outer rail to connect to.
    inline?: boolean;
    draft?: boolean;
    origin?: CommentOrigin;
  }

  const {
    thread,
    rid,
    currentUserNid,
    canModifyComment,
    editComment,
    createReply,
    reactOnComment,
    deleteComment,
    changeCommentStatus,
    canResolve = false,
    inline = false,
    draft = false,
    origin,
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
</script>

<style>
  .comments {
    display: flex;
    flex-direction: column;
    width: 100%;
    gap: 0.5rem;
  }

  .top-level-comment {
    background-color: var(--color-surface-canvas);
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
  }

  .replies-wrapper {
    position: relative;
    margin-left: 3rem;
  }
  .replies-wrapper::before,
  .replies-wrapper::after {
    content: "";
    position: absolute;
    top: -0.5rem;
    height: calc(100% + 0.5rem);
    width: 1px;
    background-color: var(--color-border-subtle);
  }
  .replies-wrapper::before {
    left: -1.75rem;
  }
  .replies-wrapper::after {
    left: 1.25rem;
  }
  /* The outer rail continues the surrounding activity timeline down to the
     top-level composer. Inline code threads have no timeline around them, so
     it would end in mid-air; only the reply indent line is drawn there. */
  .replies-wrapper.inline::before {
    content: none;
  }
  .replies-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .reply-box,
  .reply-form-box {
    position: relative;
    z-index: 1;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    background-color: var(--color-surface-canvas);
  }
  .reply-form-box {
    padding: 1rem;
  }
</style>

{#snippet repliesSnippet()}
  <div class="replies-list">
    {#each replies as reply}
      <div class="reply-box">
        <CommentComponent
          disallowEmptyBody
          {rid}
          {currentUserNid}
          lastEdit={reply.edits.length > 1 ? reply.edits.at(-1) : undefined}
          id={reply.id}
          author={reply.author}
          caption="replied"
          reactions={reply.reactions}
          timestamp={reply.edits[0].timestamp}
          body={reply.edits.slice(-1)[0].body}
          editComment={canModifyComment(reply.author.did) &&
            editComment?.bind(null, reply.id)}
          reactOnComment={reactOnComment?.bind(null, reply.id)}
          deleteComment={canModifyComment(reply.author.did)
            ? deleteComment?.bind(null, reply.id)
            : undefined} />
      </div>
    {/each}
    {#if createReply && showReplyForm}
      <div class="reply-form-box" id={`reply-${root.id}`}>
        <ExtendedTextarea
          inline
          disallowEmptyBody
          {submitInProgress}
          {rid}
          placeholder="Reply to comment"
          submitCaption="Reply"
          submitActiveVariant="secondary"
          focus
          close={() => (showReplyForm = false)}
          submit={async ({ comment, embeds }) => {
            try {
              submitInProgress = true;
              await createReply(comment, Array.from(embeds.values()), root.id);
            } finally {
              showReplyForm = false;
              submitInProgress = false;
            }
          }} />
      </div>
    {/if}
  </div>
{/snippet}

<div class="comments">
  <div class="top-level-comment">
    <CommentComponent
      disallowEmptyBody
      {rid}
      {currentUserNid}
      {draft}
      {origin}
      id={root.id}
      lastEdit={root.edits.length > 1 ? root.edits.at(-1) : undefined}
      author={root.author}
      reactions={root.reactions}
      timestamp={root.edits.slice(-1)[0].timestamp}
      body={root.edits.slice(-1)[0].body}
      editComment={canModifyComment(root.author.did) &&
        editComment?.bind(null, root.id)}
      reactOnComment={reactOnComment?.bind(null, root.id)}
      deleteComment={canModifyComment(root.author.did)
        ? deleteComment?.bind(null, root.id)
        : undefined}>
      {#snippet actions()}
        {#if changeCommentStatus && canResolve}
          <span
            class="global-icon-button"
            title={root.resolved ? "Mark as unresolved" : "Mark as resolved"}>
            <Icon
              name={root.resolved ? "comment-checkmark" : "checkmark"}
              onclick={() => changeCommentStatus(root.id, !root.resolved)} />
          </span>
        {/if}
        {#if createReply}
          <span class="global-icon-button" title="Reply">
            <Icon name="reply" onclick={toggleReply} />
          </span>
        {/if}
      {/snippet}
    </CommentComponent>
  </div>
  {#if replies.length > 0 || (createReply && showReplyForm)}
    <div class="replies-wrapper" class:inline>
      {@render repliesSnippet()}
    </div>
  {/if}
</div>
