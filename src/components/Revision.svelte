<script lang="ts">
  import type { Author } from "@bindings/cob/Author";
  import type { Config } from "@bindings/config/Config";
  import type { Diff } from "@bindings/diff/Diff";
  import type { Embed } from "@bindings/cob/thread/Embed";
  import type { Revision } from "@bindings/cob/patch/Revision";
  import type { Thread } from "@bindings/cob/thread/Thread";

  import partial from "lodash/partial";
  import { tick } from "svelte";

  import * as roles from "@app/lib/roles";
  import { announce } from "@app/components/AnnounceSwitch.svelte";
  import { invoke } from "@app/lib/invoke";
  import { nodeRunning } from "@app/lib/events";
  import { publicKeyFromDid, scrollIntoView } from "@app/lib/utils";

  import Changeset from "@app/components/Changeset.svelte";
  import CommentComponent from "@app/components/Comment.svelte";
  import CommentToggleInput from "@app/components/CommentToggleInput.svelte";
  import Icon from "@app/components/Icon.svelte";
  import ThreadComponent from "@app/components/Thread.svelte";

  interface Props {
    rid: string;
    repoDelegates: Author[];
    patchId: string;
    revision: Revision;
    config: Config;
    reload: () => Promise<void>;
  }

  /* eslint-disable prefer-const */
  let { rid, repoDelegates, patchId, revision, config, reload }: Props =
    $props();
  /* eslint-enable prefer-const */

  let focusReply: boolean = $state(false);
  let hideChanges = $state(false);
  let hideDiscussion = $state(false);
  let topLevelReplyOpen = $state(false);

  const threads = $derived(
    ((revision.discussion &&
      revision.discussion
        .filter(
          comment =>
            (comment.id !== revision.id && !comment.replyTo) ||
            comment.replyTo === revision.id,
        )
        .map(thread => {
          return {
            root: thread,
            replies:
              revision.discussion &&
              revision.discussion
                .filter(comment => comment.replyTo === thread.id)
                .sort((a, b) => a.edits[0].timestamp - b.edits[0].timestamp),
          };
        }, [])) as Thread[]) || [],
  );

  $effect(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    patchId;

    hideDiscussion = false;
    hideChanges = false;
  });

  async function editRevision(
    revisionId: string,
    description: string,
    embeds: Embed[],
  ) {
    try {
      await invoke("edit_patch", {
        rid: rid,
        cobId: patchId,
        action: {
          type: "revision.edit",
          revision: revisionId,
          description,
          embeds,
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Editing revision failed: ", error);
    } finally {
      await reload();
    }
  }

  async function reactOnRevision(
    publicKey: string,
    revisionId: string,
    authors: Author[],
    reaction: string,
  ) {
    try {
      await invoke("edit_patch", {
        rid: rid,
        cobId: patchId,
        action: {
          type: "revision.react",
          revision: revisionId,
          reaction,
          active: !authors.find(
            ({ did }) => publicKeyFromDid(did) === publicKey,
          ),
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Editing reactions failed", error);
    } finally {
      await reload();
    }
  }

  async function editComment(commentId: string, body: string, embeds: Embed[]) {
    try {
      await invoke("edit_patch", {
        rid: rid,
        cobId: patchId,
        action: {
          type: "revision.comment.edit",
          comment: commentId,
          body,
          revision: revision.id,
          embeds,
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Eediting comment failed: ", error);
    } finally {
      await reload();
    }
  }

  async function createReply(replyTo: string, body: string, embeds: Embed[]) {
    try {
      await invoke("create_patch_comment", {
        rid: rid,
        new: { id: patchId, body, embeds, replyTo, revision: revision.id },
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
          revision: revision.id,
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
        new: { id: patchId, body, embeds, revision: revision.id },
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
    focusReply = true;
  }

  async function loadHighlightedDiff(rid: string, base: string, head: string) {
    return invoke<Diff>("get_diff", {
      rid,
      options: {
        base,
        head,
        unified: 5,
        highlight: true,
      },
    });
  }
</script>

<style>
  .patch-body {
    margin-bottom: 1rem;
    position: relative;
    z-index: 1;
  }
  /* We put the background and clip-path in a separate element to prevent
     popovers being clipped in the main element. */
  .patch-body::after {
    position: absolute;
    z-index: -1;
    content: " ";
    background-color: var(--color-background-float);
    clip-path: var(--2px-corner-fill);
    width: 100%;
    height: 100%;
    top: 0;
  }
  .hide {
    display: none;
  }
  .connector {
    width: 2px;
    height: 1rem;
    margin-left: 1.25rem;
    background-color: var(--color-background-float);
  }
</style>

<div class="txt-small patch-body">
  <CommentComponent
    caption="opened"
    {rid}
    id={patchId}
    lastEdit={revision.description.length > 1
      ? revision.description.at(-1)
      : undefined}
    author={revision.author}
    reactions={revision.reactions}
    timestamp={revision.timestamp}
    body={revision.description.slice(-1)[0].body}
    reactOnComment={partial(reactOnRevision, config.publicKey, revision.id)}
    editComment={roles.isDelegateOrAuthor(
      config.publicKey,
      repoDelegates.map(delegate => delegate.did),
      revision.author.did,
    ) && partial(editRevision, revision.id)}>
    {#snippet actions()}
      <Icon name="reply" onclick={toggleReply} />
    {/snippet}
  </CommentComponent>
</div>

<div style:margin="1rem 0">
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div
    role="button"
    tabindex="0"
    class="txt-semibold global-flex"
    style:margin-bottom="1rem"
    style:cursor="pointer"
    onclick={() => (hideDiscussion = !hideDiscussion)}>
    <Icon name={hideDiscussion ? "chevron-right" : "chevron-down"} />Discussion
  </div>
  <div class:hide={hideDiscussion}>
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
          ? () => (topLevelReplyOpen = false)
          : undefined}
        placeholder="Leave a comment"
        submit={partial(createComment)} />
    </div>
  </div>
</div>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  role="button"
  tabindex="0"
  class="txt-semibold global-flex"
  style:margin-bottom={hideChanges ? undefined : "1rem"}
  style:cursor="pointer"
  onclick={() => (hideChanges = !hideChanges)}>
  <Icon name={hideChanges ? "chevron-right" : "chevron-down"} />Changes
</div>
<div class:hide={hideChanges}>
  {#await loadHighlightedDiff(rid, revision.base, revision.head)}
    <span class="txt-small">Loading…</span>
  {:then diff}
    <Changeset {diff} repoId={rid} />
  {/await}
</div>
