<script lang="ts">
  import type { Author } from "@bindings/cob/Author";
  import type { Revision } from "@bindings/cob/patch/Revision";
  import type { Embed } from "@bindings/cob/thread/Embed";
  import type { Thread } from "@bindings/cob/thread/Thread";
  import type { Config } from "@bindings/config/Config";

  import { nodeRunning } from "@app/lib/events";
  import { invoke } from "@app/lib/invoke";
  import * as roles from "@app/lib/roles";
  import { publicKeyFromDid } from "@app/lib/utils";

  import { announce } from "@app/components/AnnounceSwitch.svelte";
  import Changes from "@app/components/Changes.svelte";
  import CommentComponent from "@app/components/Comment.svelte";
  import Discussion from "@app/components/Discussion.svelte";

  interface Props {
    rid: string;
    repoDelegates: Author[];
    patchId: string;
    revision: Revision;
    config: Config;
    loadPatch: () => Promise<void>;
    hideDescription?: boolean;
  }

  const {
    rid,
    repoDelegates,
    patchId,
    revision,
    config,
    loadPatch,
    hideDescription = false,
  }: Props = $props();

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

  async function editRevision(description: string, embeds: Embed[]) {
    try {
      await invoke("edit_patch", {
        rid: rid,
        cobId: patchId,
        action: {
          type: "revision.edit",
          revision: revision.id,
          description,
          embeds,
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Editing revision failed: ", error);
    } finally {
      await loadPatch();
    }
  }

  async function reactOnRevision(authors: Author[], reaction: string) {
    try {
      await invoke("edit_patch", {
        rid: rid,
        cobId: patchId,
        action: {
          type: "revision.react",
          revision: revision.id,
          reaction,
          active: !authors.find(
            ({ did }) => publicKeyFromDid(did) === config.publicKey,
          ),
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Editing reactions failed", error);
    } finally {
      await loadPatch();
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
      await loadPatch();
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
      await loadPatch();
    }
  }

  async function reactOnComment(
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
            ({ did }) => publicKeyFromDid(did) === config.publicKey,
          ),
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Editing comment reactions failed", error);
    } finally {
      await loadPatch();
    }
  }
</script>

<style>
  .patch-body {
    margin-bottom: 1rem;
    background-color: var(--color-surface-canvas);
    border-radius: var(--border-radius-sm);
  }
</style>

{#if !hideDescription}
  <div class="txt-body-m-regular patch-body">
    <CommentComponent
      caption={revision.id === patchId ? "opened patch" : "created revision"}
      {rid}
      currentUserNid={config.publicKey}
      id={revision.id}
      lastEdit={revision.description.length > 1
        ? revision.description.at(-1)
        : undefined}
      author={revision.author}
      reactions={revision.reactions}
      timestamp={revision.timestamp}
      body={revision.description.slice(-1)[0].body}
      reactOnComment={reactOnRevision}
      editComment={roles.isDelegateOrAuthor(
        config.publicKey,
        repoDelegates.map(delegate => delegate.did),
        revision.author.did,
      ) && editRevision}>
    </CommentComponent>
  </div>
{/if}

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
