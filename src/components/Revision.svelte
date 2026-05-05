<script lang="ts">
  import type { CodeComments } from "@app/components/Diff.svelte";
  import type { Author } from "@bindings/cob/Author";
  import type { Operation } from "@bindings/cob/Operation";
  import type { Action } from "@bindings/cob/patch/Action";
  import type { Revision } from "@bindings/cob/patch/Revision";
  import type { CodeLocation } from "@bindings/cob/thread/CodeLocation";
  import type { Embed } from "@bindings/cob/thread/Embed";
  import type { Thread } from "@bindings/cob/thread/Thread";
  import type { Config } from "@bindings/config/Config";
  import type { Commit } from "@bindings/repo/Commit";

  import { draftReviewStorage } from "@app/lib/draftReviewStorage";
  import { nodeRunning } from "@app/lib/events";
  import { cachedListCommits, invoke } from "@app/lib/invoke";
  import * as roles from "@app/lib/roles";
  import { didFromPublicKey, publicKeyFromDid } from "@app/lib/utils";

  import { announce } from "@app/components/AnnounceSwitch.svelte";
  import Changes from "@app/components/Changes.svelte";
  import CommentComponent from "@app/components/Comment.svelte";
  import CommitActivityItem from "@app/components/CommitActivityItem.svelte";
  import CommitGroupActivityItem from "@app/components/CommitGroupActivityItem.svelte";
  import Discussion, {
    type ActivityItem,
  } from "@app/components/Discussion.svelte";
  import PatchActivityItem, {
    type FlattenedPatchOperation,
  } from "@app/components/PatchActivityItem.svelte";
  import ReviewCodeThread from "@app/components/ReviewCodeThread.svelte";

  type ActivityData =
    | { kind: "op"; op: FlattenedPatchOperation }
    | { kind: "commit"; commit: Commit }
    | { kind: "commitGroup"; groupId: string; commits: Commit[] }
    | { kind: "reviewCode"; thread: Thread<CodeLocation> };

  const COMMIT_COLLAPSE_THRESHOLD = 5;

  interface Props {
    rid: string;
    repoDelegates: Author[];
    patchId: string;
    revision: Revision;
    config: Config;
    loadPatch: () => Promise<void>;
    view?: "description" | "activity" | "changes";
    activity?: Operation<Action>[];
    revisions?: Revision[];
  }

  const {
    rid,
    repoDelegates,
    patchId,
    revision,
    config,
    loadPatch,
    view = "activity",
    activity = [],
    revisions = [],
  }: Props = $props();
  const currentUserAuthor: Author = $derived({
    did: didFromPublicKey(config.publicKey),
    alias: config.alias ?? undefined,
  });

  const draftReview = $derived(
    draftReviewStorage.getForRevision(revision.id, currentUserAuthor),
  );

  const hasPublishedReview = $derived(
    revision.reviews?.some(r => r.author.did === currentUserAuthor.did) ??
      false,
  );

  const codeCommentThreads: Thread<CodeLocation>[] = $derived(
    draftReview
      ? (draftReview.comments
          .filter(c => c.location && !c.replyTo)
          .map(root => ({
            root,
            replies: draftReview.comments.filter(c => c.replyTo === root.id),
          })) as Thread<CodeLocation>[])
      : [],
  );

  async function createCodeComment(
    body: string,
    embeds: Embed[],
    _replyTo?: string,
    location?: CodeLocation,
  ) {
    if (!location) return;
    try {
      let draftId = draftReview?.id;
      if (!draftId) {
        draftId = draftReviewStorage.create(rid, patchId, revision.id);
      }
      draftReviewStorage.addComment(draftId, { body, embeds, location });
    } catch (error) {
      console.error("Creating code comment failed", error);
    } finally {
      await loadPatch();
    }
  }

  async function editCodeComment(
    commentId: string,
    body: string,
    embeds: Embed[],
  ) {
    if (!draftReview) return;
    draftReviewStorage.updateComment(draftReview.id, commentId, {
      body,
      embeds,
    });
    await loadPatch();
  }

  async function deleteCodeComment(commentId: string) {
    if (!draftReview) return;
    draftReviewStorage.deleteComment(draftReview.id, commentId);
    await loadPatch();
  }

  const codeComments: CodeComments | undefined = $derived(
    hasPublishedReview
      ? undefined
      : {
          config,
          createComment: createCodeComment,
          editComment: editCodeComment,
          deleteComment: deleteCodeComment,
          repoDelegates,
          rid,
          threads: codeCommentThreads,
          canReply: true,
          disableAttachments: "Publish your review to attach files",
        },
  );

  let commitsByRevision: Record<string, Commit[]> = $state({});

  $effect(() => {
    const ridLocal = rid;
    const revs = [...revisions].sort((a, b) => a.timestamp - b.timestamp);
    void Promise.all(
      revs.map(async (rev, idx): Promise<[string, Commit[]]> => {
        const prev = revs[idx - 1];
        const base = prev ? prev.head : rev.base;
        try {
          const commits = await cachedListCommits(ridLocal, base, rev.head);
          return [rev.id, commits];
        } catch (error) {
          console.error(
            `Failed to load commits for revision ${rev.id} (${base}..${rev.head})`,
            error,
          );
          return [rev.id, []];
        }
      }),
    ).then(entries => {
      const next: Record<string, Commit[]> = {};
      entries.forEach(([id, commits]) => {
        next[id] = commits;
      });
      commitsByRevision = next;
    });
  });
  const skippedActivityTypes = new Set<Action["type"]>([
    "revision.comment",
    "revision.comment.edit",
    "revision.comment.redact",
    "revision.comment.react",
    "revision.react",
    "revision.edit",
    "revision.redact",
    "review.comment",
    "review.comment.edit",
    "review.comment.redact",
    "review.comment.react",
    "review.comment.resolve",
    "review.comment.unresolve",
    "review.edit",
    "review.redact",
    "review.react",
  ]);

  const activityItems: ActivityItem<ActivityData>[] = $derived.by(() => {
    const tracker: Partial<Record<Action["type"], Action>> = {};
    const items: ActivityItem<ActivityData>[] = [];
    activity.forEach(operation => {
      operation.actions.forEach((action, actionIndex) => {
        if (skippedActivityTypes.has(action.type)) {
          tracker[action.type] = action;
          return;
        }
        const previous = tracker[action.type];
        // The first `edit` action has nothing to diff against, so the
        // renderer skips it. Skip it here too so we don't leave a gap.
        if (action.type === "edit" && !previous) {
          tracker[action.type] = action;
          return;
        }
        const op: FlattenedPatchOperation = {
          ...action,
          id: operation.id,
          author: operation.author,
          timestamp: operation.timestamp,
          previous,
        };
        tracker[action.type] = action;
        items.push({
          key: `${operation.id}:${actionIndex}`,
          timestamp: operation.timestamp,
          data: { kind: "op", op },
        });
      });
    });

    (revision.reviews ?? []).forEach(review => {
      const reviewComments = review.comments ?? [];
      reviewComments
        .filter(c => c.location && !c.replyTo && c.id !== review.id)
        .forEach(root => {
          const replies = reviewComments
            .filter(c => c.replyTo === root.id)
            .sort((a, b) => a.edits[0].timestamp - b.edits[0].timestamp);
          const thread = { root, replies } as Thread<CodeLocation>;
          items.push({
            key: `review:${review.id}:${root.id}`,
            timestamp: root.edits[0].timestamp,
            data: { kind: "reviewCode", thread },
          });
        });
    });

    const sortedRevs = [...revisions].sort((a, b) => a.timestamp - b.timestamp);
    const patchOpenTimestamp = sortedRevs[0]?.timestamp ?? 0;
    const seenCommitIds = new Set<string>();
    Object.values(commitsByRevision).forEach(commits => {
      commits.forEach(commit => {
        if (seenCommitIds.has(commit.id)) {
          return;
        }
        seenCommitIds.add(commit.id);
        const timestampMs = commit.committer.time * 1000;
        if (timestampMs < patchOpenTimestamp) {
          return;
        }
        items.push({
          key: `commit:${commit.id}`,
          timestamp: timestampMs,
          data: { kind: "commit", commit },
        });
      });
    });

    items.sort((a, b) => a.timestamp - b.timestamp);

    const grouped: ActivityItem<ActivityData>[] = [];
    let i = 0;
    while (i < items.length) {
      if (items[i].data.kind !== "commit") {
        grouped.push(items[i]);
        i++;
        continue;
      }
      let j = i;
      while (j < items.length && items[j].data.kind === "commit") {
        j++;
      }
      const runLength = j - i;
      if (runLength > COMMIT_COLLAPSE_THRESHOLD) {
        const commits = items.slice(i, j).map(item => {
          if (item.data.kind !== "commit") {
            throw new Error("unreachable");
          }
          return item.data.commit;
        });
        const groupId = `commit-group:${commits[0].id}:${commits[commits.length - 1].id}`;
        grouped.push({
          key: groupId,
          timestamp: items[i].timestamp,
          data: { kind: "commitGroup", groupId, commits },
        });
      } else {
        for (let k = i; k < j; k++) {
          grouped.push(items[k]);
        }
      }
      i = j;
    }

    return grouped;
  });
  const reviewSummaryFingerprints = $derived(
    new Set(
      (revision.reviews ?? [])
        .filter(r => r.summary && r.summary.trim() !== "")
        .map(r => `${r.author.did} ${r.summary}`),
    ),
  );
  const commentThreads = $derived(
    ((revision.discussion &&
      revision.discussion
        .filter(
          comment =>
            (comment.id !== revision.id && !comment.replyTo) ||
            comment.replyTo === revision.id,
        )
        .filter(comment => {
          const body = comment.edits[comment.edits.length - 1]?.body ?? "";
          return !reviewSummaryFingerprints.has(
            `${comment.author.did} ${body}`,
          );
        })
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

{#if view === "description"}
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
{:else if view === "activity"}
  {#snippet renderActivity(data: ActivityData)}
    {#if data.kind === "op"}
      <PatchActivityItem op={data.op} {patchId} />
    {:else if data.kind === "commit"}
      <CommitActivityItem commit={data.commit} {rid} />
    {:else if data.kind === "commitGroup"}
      <CommitGroupActivityItem commits={data.commits} {rid} />
    {:else}
      <ReviewCodeThread
        {rid}
        base={revision.base}
        head={revision.head}
        thread={data.thread}
        {config}
        {repoDelegates} />
    {/if}
  {/snippet}

  <Discussion
    cobId={patchId}
    {commentThreads}
    {config}
    {createComment}
    {editComment}
    {reactOnComment}
    {repoDelegates}
    {rid}
    {activityItems}
    {renderActivity} />
{:else}
  <Changes {rid} {patchId} {revision} {codeComments} />
{/if}
