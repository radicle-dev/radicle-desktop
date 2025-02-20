<script lang="ts">
  import type { Author } from "@bindings/cob/Author";
  import type { Config } from "@bindings/config/Config";
  import type { Embed } from "@bindings/cob/thread/Embed";
  import type { PatchStatus } from "@app/views/repo/router";
  import type { Revision } from "@bindings/cob/patch/Revision";
  import type { Thread } from "@bindings/cob/thread/Thread";

  import partial from "lodash/partial";

  import * as roles from "@app/lib/roles";
  import { announce } from "@app/components/AnnounceSwitch.svelte";
  import { invoke } from "@app/lib/invoke";
  import { nodeRunning } from "@app/lib/events";
  import { publicKeyFromDid } from "@app/lib/utils";

  import Changes from "@app/components/Changes.svelte";
  import CommentComponent from "@app/components/Comment.svelte";
  import Discussion from "@app/components/Discussion.svelte";
  import Reviews from "@app/components/Reviews.svelte";

  interface Props {
    rid: string;
    repoDelegates: Author[];
    patchId: string;
    revision: Revision;
    config: Config;
    status: PatchStatus | undefined;
    reload: () => Promise<void>;
  }

  /* eslint-disable prefer-const */
  let { rid, repoDelegates, patchId, revision, config, status, reload }: Props =
    $props();
  /* eslint-enable prefer-const */

  const commentThreads = $derived(
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

  async function createComment(
    body: string,
    embeds: Embed[],
    replyTo?: string,
  ) {
    try {
      await invoke("create_patch_comment", {
        rid: rid,
        new: { id: patchId, body, embeds, replyTo, revision: revision.id },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Creating comment failed", error);
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
      console.error("Editing comment failed: ", error);
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
</style>

<div class="txt-small patch-body">
  <CommentComponent
    caption={revision.id === patchId ? "opened patch" : "created revision"}
    {rid}
    id={revision.id}
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
  </CommentComponent>
</div>

<Reviews {config} {patchId} {reload} {revision} {rid} {status} />

<Discussion
  cobId={patchId}
  {commentThreads}
  {config}
  {createComment}
  {editComment}
  {reactOnComment}
  {repoDelegates}
  {rid} />

<Changes {rid} {patchId} {revision} />
