<script lang="ts">
  import type { Author } from "@bindings/cob/Author";
  import type { Embed } from "@bindings/cob/thread/Embed";
  import type { Thread } from "@bindings/cob/thread/Thread";
  import type { Config } from "@bindings/config/Config";

  import partial from "lodash/partial";
  import sum from "lodash/sum";

  import * as roles from "@app/lib/roles";

  import Button from "@app/components/Button.svelte";
  import ExtendedTextarea from "@app/components/ExtendedTextarea.svelte";
  import Icon from "@app/components/Icon.svelte";
  import ThreadComponent from "@app/components/Thread.svelte";

  interface Props {
    cobId: string;
    commentThreads: Thread[];
    config: Config;
    createComment: (
      body: string,
      embeds: Embed[],
      replyTo?: string,
    ) => Promise<void>;
    editComment: (
      commentId: string,
      body: string,
      embeds: Embed[],
    ) => Promise<void>;
    reactOnComment: (
      commentId: string,
      authors: Author[],
      reaction: string,
    ) => Promise<void>;
    repoDelegates: Author[];
    rid: string;
  }

  /* eslint-disable prefer-const */
  let {
    cobId,
    commentThreads,
    config,
    createComment,
    editComment,
    reactOnComment,
    repoDelegates,
    rid,
  }: Props = $props();
  /* eslint-enable prefer-const */

  let previousCobId = cobId;
  let focusReply: boolean = $state(false);
  let commentFormKey = $state(0);

  let hideDiscussion = $state(false);

  $effect(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    cobId;

    if (cobId !== previousCobId) {
      previousCobId = cobId;
      hideDiscussion = false;
      focusReply = false;
      commentFormKey += 1;
    }
  });
</script>

<style>
  .connector {
    width: 1px;
    height: 1rem;
    margin-left: 1.25rem;
    background-color: var(--color-border-subtle);
  }
</style>

<div style:margin={hideDiscussion ? "1.5rem 0" : "1.5rem 0 2.5rem 0"}>
  <div class="global-flex">
    <div class="global-flex">
      <Button
        variant="naked"
        disabled={commentThreads.length === 0}
        onclick={() => (hideDiscussion = !hideDiscussion)}>
        <Icon name={hideDiscussion ? "chevron-right" : "chevron-down"} />
      </Button>
      <div
        class="txt-body-m-regular global-flex"
        style:color={commentThreads.length === 0
          ? "var(--color-text-disabled)"
          : undefined}>
        Discussion <span>
          {sum(
            commentThreads.map(t => {
              return t.replies.length + 1;
            }),
          )}
        </span>
      </div>
    </div>
  </div>
  <div
    style:display={hideDiscussion ? "none" : "revert"}
    style:margin-top="1rem">
    {#each commentThreads as thread}
      <ThreadComponent
        {thread}
        {rid}
        canEditComment={partial(
          roles.isDelegateOrAuthor,
          config.publicKey,
          repoDelegates.map(delegate => delegate.did),
        )}
        {editComment}
        createReply={createComment}
        {reactOnComment} />
      <div class="connector"></div>
    {/each}

    <div id={`reply-${cobId}`}>
      {#key commentFormKey}
        <ExtendedTextarea
          disallowEmptyBody
          {rid}
          focus={focusReply}
          borderVariant="ghost"
          stylePadding="0.5rem 0.75rem"
          hideDiscard
          placeholder="Leave a comment"
          submitActiveVariant="secondary"
          close={() => {
            focusReply = false;
            commentFormKey += 1;
          }}
          submit={async ({ comment, embeds }) => {
            try {
              await createComment(comment, Array.from(embeds.values()));
            } finally {
              focusReply = false;
              commentFormKey += 1;
            }
          }} />
      {/key}
    </div>
  </div>
</div>
