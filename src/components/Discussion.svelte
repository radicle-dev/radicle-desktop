<script lang="ts">
  import type { Author } from "@bindings/cob/Author";
  import type { Config } from "@bindings/config/Config";
  import type { Embed } from "@bindings/cob/thread/Embed";
  import type { Thread } from "@bindings/cob/thread/Thread";

  import partial from "lodash/partial";
  import sum from "lodash/sum";
  import { tick } from "svelte";

  import * as roles from "@app/lib/roles";
  import { scrollIntoView } from "@app/lib/utils";

  import CommentToggleInput from "@app/components/CommentToggleInput.svelte";
  import Icon from "@app/components/Icon.svelte";
  import NakedButton from "./NakedButton.svelte";
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
      publicKey: string,
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

  $effect(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    cobId;

    hideDiscussion = commentThreads.length === 0;
    focusReply = false;
    topLevelReplyOpen = false;
  });

  let focusReply: boolean = $state(false);
  let topLevelReplyOpen = $state(false);

  let hideDiscussion = $state(commentThreads.length === 0);

  async function toggleReply() {
    topLevelReplyOpen = !topLevelReplyOpen;
    if (!topLevelReplyOpen) {
      return;
    }

    await tick();
    scrollIntoView(`reply-${cobId}`, {
      behavior: "smooth",
      block: "center",
    });
    await tick();
    focusReply = true;
  }
</script>

<style>
  .connector {
    width: 2px;
    height: 1rem;
    margin-left: 1.25rem;
    background-color: var(--color-background-float);
  }
</style>

<div style:margin={hideDiscussion ? "1.5rem 0" : "1.5rem 0 2.5rem 0"}>
  <div class="global-flex">
    <NakedButton
      variant="ghost"
      disabled={commentThreads.length === 0}
      onclick={() => (hideDiscussion = !hideDiscussion)}>
      <Icon name={hideDiscussion ? "chevron-right" : "chevron-down"} />
      <div class="txt-semibold global-flex txt-regular">
        Discussion <span style:font-weight="var(--font-weight-regular)">
          {sum(
            commentThreads.map(t => {
              return t.replies.length + 1;
            }),
          )}
        </span>
      </div>
    </NakedButton>
    <div style:margin-left="auto">
      <NakedButton
        variant="ghost"
        onclick={async () => {
          if (hideDiscussion) {
            hideDiscussion = false;
          } else {
            if (commentThreads.length === 0) {
              hideDiscussion = true;
            }
          }
          await toggleReply();
        }}>
        <Icon name="comment" />
        <span class="txt-small">Comment</span>
      </NakedButton>
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
        reactOnComment={partial(reactOnComment, config.publicKey)} />
      <div class="connector"></div>
    {/each}

    <div id={`reply-${cobId}`}>
      <CommentToggleInput
        disallowEmptyBody
        {rid}
        focus={focusReply}
        onexpand={toggleReply}
        onclose={topLevelReplyOpen
          ? () => {
              if (commentThreads.length === 0) {
                hideDiscussion = !hideDiscussion;
              }
              topLevelReplyOpen = false;
            }
          : undefined}
        placeholder="Leave a comment"
        submit={createComment} />
    </div>
  </div>
</div>
