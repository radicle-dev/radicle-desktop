<script lang="ts">
  import type { Author } from "@bindings/cob/Author";
  import type { CodeLocation } from "@bindings/cob/thread/CodeLocation";
  import type { Comment } from "@bindings/cob/thread/Comment";
  import type { Config } from "@bindings/config/Config";
  import type { Embed } from "@bindings/cob/thread/Embed";
  import type { Thread } from "@bindings/cob/thread/Thread";

  import { tick } from "svelte";
  import partial from "lodash/partial";

  import * as roles from "@app/lib/roles";
  import { announce } from "@app/components/AnnounceSwitch.svelte";
  import { invoke } from "@app/lib/invoke";
  import { nodeRunning } from "@app/lib/events";
  import { publicKeyFromDid, scrollIntoView } from "@app/lib/utils";

  import CommentToggleInput from "@app/components/CommentToggleInput.svelte";
  import Icon from "@app/components/Icon.svelte";
  import NakedButton from "./NakedButton.svelte";
  import ThreadComponent from "@app/components/Thread.svelte";

  interface Props {
    config: Config;
    discussion?: Array<Comment<CodeLocation>>;
    patchId: string;
    reload: () => Promise<void>;
    repoDelegates: Author[];
    revisionId: string;
    rid: string;
  }

  /* eslint-disable prefer-const */
  let {
    config,
    discussion,
    patchId,
    reload,
    repoDelegates,
    revisionId,
    rid,
  }: Props = $props();
  /* eslint-enable prefer-const */

  $effect(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    patchId;

    hideDiscussion = discussion === undefined || discussion.length === 0;
    focusReply = false;
    topLevelReplyOpen = false;
  });

  let focusReply: boolean = $state(false);
  let topLevelReplyOpen = $state(false);

  let hideDiscussion = $state(
    discussion === undefined || discussion.length === 0,
  );

  const threads = $derived(
    ((discussion &&
      discussion
        .filter(
          comment =>
            (comment.id !== revisionId && !comment.replyTo) ||
            comment.replyTo === revisionId,
        )
        .map(thread => {
          return {
            root: thread,
            replies:
              discussion &&
              discussion
                .filter(comment => comment.replyTo === thread.id)
                .sort((a, b) => a.edits[0].timestamp - b.edits[0].timestamp),
          };
        }, [])) as Thread[]) || [],
  );

  async function editComment(commentId: string, body: string, embeds: Embed[]) {
    try {
      await invoke("edit_patch", {
        rid: rid,
        cobId: patchId,
        action: {
          type: "revision.comment.edit",
          comment: commentId,
          body,
          revision: revisionId,
          embeds,
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Editing comment failed: ", error);
    } finally {
      await reload();
    }
  }

  async function createReply(replyTo: string, body: string, embeds: Embed[]) {
    try {
      await invoke("create_patch_comment", {
        rid: rid,
        new: { id: patchId, body, embeds, replyTo, revision: revisionId },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Creating reply failed", error);
    } finally {
      await reload();
    }
  }

  async function reactOnComment(
    publicKey: string,
    commentId: string,
    authors: Author[],
    reaction: string,
  ) {
    try {
      await invoke("edit_patch", {
        rid: rid,
        cobId: patchId,
        action: {
          type: "revision.comment.react",
          comment: commentId,
          reaction,
          revision: revisionId,
          active: !authors.find(
            ({ did }) => publicKeyFromDid(did) === publicKey,
          ),
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Editing comment reactions failed", error);
    } finally {
      await reload();
    }
  }

  async function createComment(body: string, embeds: Embed[]) {
    try {
      await invoke("create_patch_comment", {
        rid: rid,
        new: { id: patchId, body, embeds, revision: revisionId },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Creating comment failed: ", error);
    } finally {
      await reload();
    }
  }

  async function toggleReply() {
    topLevelReplyOpen = !topLevelReplyOpen;
    if (!topLevelReplyOpen) {
      return;
    }

    await tick();
    scrollIntoView(`reply-${patchId}`, {
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
  .hide {
    display: none;
  }
</style>

<div style:margin={hideDiscussion ? "1.5rem 0" : "0 0 2.5rem 0"}>
  <div class="global-flex">
    <NakedButton
      variant="ghost"
      disabled={discussion === undefined || discussion.length === 0}
      onclick={() => (hideDiscussion = !hideDiscussion)}>
      <Icon name={hideDiscussion ? "chevron-right" : "chevron-down"} />
      <div class="txt-semibold global-flex txt-regular">
        Discussion <span style:font-weight="var(--font-weight-regular)">
          {discussion?.length ?? 0}
        </span>
      </div>
    </NakedButton>
    <div style:margin-left="auto">
      <NakedButton
        variant="secondary"
        onclick={async () => {
          if (hideDiscussion) {
            hideDiscussion = false;
          } else {
            if (discussion === undefined || discussion.length === 0) {
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
  <div class:hide={hideDiscussion} style:margin-top="1rem">
    {#each threads as thread}
      <ThreadComponent
        {thread}
        {rid}
        canEditComment={partial(
          roles.isDelegateOrAuthor,
          config.publicKey,
          repoDelegates.map(delegate => delegate.did),
        )}
        editComment={partial(editComment)}
        createReply={partial(createReply)}
        reactOnComment={partial(reactOnComment, config.publicKey)} />
      <div class="connector"></div>
    {/each}

    <div id={`reply-${patchId}`}>
      <CommentToggleInput
        disallowEmptyBody
        {rid}
        focus={focusReply}
        onexpand={toggleReply}
        onclose={topLevelReplyOpen
          ? () => {
              if (discussion === undefined || discussion.length === 0) {
                hideDiscussion = !hideDiscussion;
              }
              topLevelReplyOpen = false;
            }
          : undefined}
        placeholder="Leave a comment"
        submit={partial(createComment)} />
    </div>
  </div>
</div>
