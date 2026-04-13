<script lang="ts">
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
    deleteComment?: (commentId: string) => Promise<void>;
    inline?: boolean;
  }

  const {
    thread,
    rid,
    canEditComment,
    editComment,
    createReply,
    reactOnComment,
    deleteComment,
    inline = false,
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
      ? `--local-border-radius: var(--border-radius-sm) var(--border-radius-sm) 0 0`
      : `--local-border-radius: var(--border-radius-sm)`,
  );
</script>

<style>
  .comments {
    display: flex;
    flex-direction: column;
    width: 100%;
  }

  .top-level-comment {
    background-color: var(--color-surface-canvas);
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--local-border-radius);
  }
</style>

{#snippet repliesSnippet()}
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
          editComment={canEditComment(reply.author.did) &&
            editComment?.bind(null, reply.id)}
          reactOnComment={reactOnComment?.bind(null, reply.id)} />
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

<div class="comments" {style}>
  <div class:top-level-comment={!inline}>
    <CommentComponent
      disallowEmptyBody
      {rid}
      id={root.id}
      lastEdit={root.edits.length > 1 ? root.edits.at(-1) : undefined}
      author={root.author}
      reactions={root.reactions}
      timestamp={root.edits.slice(-1)[0].timestamp}
      body={root.edits.slice(-1)[0].body}
      editComment={canEditComment(root.author.did) &&
        editComment?.bind(null, root.id)}
      reactOnComment={reactOnComment?.bind(null, root.id)}
      deleteComment={deleteComment?.bind(null, root.id)}>
      {#snippet actions()}
        {#if createReply}
          <Icon name="reply" onclick={toggleReply} />
        {/if}
      {/snippet}
    </CommentComponent>
  </div>
  {#if replies.length > 0 || (createReply && showReplyForm)}
    {#if inline}
      <div
        style:background-color="var(--color-surface-canvas)"
        style:border="1px solid var(--color-border-subtle)">
        {@render repliesSnippet()}
      </div>
    {:else}
      <div
        style:border="1px solid var(--color-border-subtle)"
        style:border-top="none"
        style:border-radius="var(--border-radius-sm)"
        style:border-top-left-radius={!inline ? "0" : undefined}
        style:border-top-right-radius={!inline ? "0" : undefined}
        style:display="flex"
        style:gap="0.5rem"
        style:align-items="center"
        style:background-color="var(--color-surface-canvas)"
        style:overflow="hidden">
        {@render repliesSnippet()}
      </div>
    {/if}
  {/if}
</div>
