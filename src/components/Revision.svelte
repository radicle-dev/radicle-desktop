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
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import { slide } from "svelte/transition";

  import { draftReviewStorage } from "@app/lib/draftReviewStorage";
  import { nodeRunning } from "@app/lib/events";
  import { isIgnoredFile } from "@app/lib/ignoredFiles";
  import { cachedGetDiff, cachedListCommits, invoke } from "@app/lib/invoke";
  import * as roles from "@app/lib/roles";
  import { push } from "@app/lib/router";
  import {
    didFromPublicKey,
    pluralize,
    publicKeyFromDid,
  } from "@app/lib/utils";

  import { announce } from "@app/components/AnnounceSwitch.svelte";
  import Changes from "@app/components/Changes.svelte";
  import CommitActivityItem from "@app/components/CommitActivityItem.svelte";
  import Discussion, {
    type ActivityItem,
  } from "@app/components/Discussion.svelte";
  import ExtendedTextarea from "@app/components/ExtendedTextarea.svelte";
  import FileDiff from "@app/components/FileDiff.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Markdown from "@app/components/Markdown.svelte";
  import PatchActivityItem, {
    type FlattenedPatchOperation,
    splitDescription,
  } from "@app/components/PatchActivityItem.svelte";
  import { closeFocused } from "@app/components/Popover.svelte";
  import Reactions from "@app/components/Reactions.svelte";
  import ReactionSelector from "@app/components/ReactionSelector.svelte";
  import ReviewCodeThread from "@app/components/ReviewCodeThread.svelte";

  type ActivityData =
    | {
        kind: "op";
        op: FlattenedPatchOperation;
        commits?: Commit[];
        reviewThreads?: Thread<CodeLocation>[];
      }
    | {
        kind: "opened";
        op: FlattenedPatchOperation & { type: "revision" };
        openedAsDraft: boolean;
      }
    | {
        kind: "olderRevisions";
        count: number;
        author?: Author;
        expanded: boolean;
      };

  interface Props {
    rid: string;
    repo: RepoInfo;
    repoDelegates: Author[];
    patchId: string;
    revision: Revision;
    config: Config;
    loadPatch: () => Promise<void>;
    view?: "description" | "activity" | "changes";
    activity?: Operation<Action>[];
    revisions?: Revision[];
    draftReviewId?: string;
    filesExpanded?: boolean;
    onMerge?: () => Promise<void> | void;
    mergeDisabledReason?: string;
    onViewChanges?: (revisionId: string) => void;
  }

  /* eslint-disable prefer-const */
  let {
    rid,
    repo,
    repoDelegates,
    patchId,
    revision,
    config,
    loadPatch,
    view = "activity",
    activity = [],
    revisions = [],
    draftReviewId,
    filesExpanded = $bindable(true),
    onMerge,
    mergeDisabledReason,
    onViewChanges,
  }: Props = $props();
  /* eslint-enable prefer-const */
  const currentUserAuthor: Author = $derived({
    did: didFromPublicKey(config.publicKey),
    alias: config.alias ?? undefined,
  });

  const latestRevisionId = $derived(
    [...revisions].sort((a, b) => b.timestamp - a.timestamp)[0]?.id,
  );
  const firstRevisionId = $derived(
    [...revisions].sort((a, b) => a.timestamp - b.timestamp)[0]?.id,
  );
  // The patch was opened as a draft when its first lifecycle change is a draft
  // happening right at creation (the first or second operation), as opposed to
  // being converted to draft later in its life.
  const openingDraftOpId = $derived.by(() => {
    const ops = [...activity].sort((a, b) => a.timestamp - b.timestamp);
    const firstLifecycleIdx = ops.findIndex(op =>
      op.actions.some(a => a.type === "lifecycle"),
    );
    if (firstLifecycleIdx === -1 || firstLifecycleIdx > 1) return undefined;
    const op = ops[firstLifecycleIdx];
    const lifecycle = op.actions.find(a => a.type === "lifecycle");
    return lifecycle?.type === "lifecycle" && lifecycle.state.status === "draft"
      ? op.id
      : undefined;
  });
  const targetBranch = $derived(
    repo.payloads["xyz.radicle.project"]?.data.defaultBranch,
  );
  let revisionToggles: Record<string, boolean> = $state({});
  let reviewToggles: Record<string, boolean> = $state({});
  let commitGroupToggles: Record<string, boolean> = $state({});
  let olderRevisionsExpanded = $state(false);
  // svelte-ignore state_referenced_locally
  let lastPatchIdSeen = patchId;
  $effect(() => {
    if (patchId !== lastPatchIdSeen) {
      lastPatchIdSeen = patchId;
      revisionToggles = {};
      reviewToggles = {};
      commitGroupToggles = {};
      olderRevisionsExpanded = false;
    }
  });
  const MAX_COMMITS_VISIBLE = 3;
  const COMMIT_COLLAPSE_THRESHOLD = 5;
  function isCommitGroupExpanded(groupKey: string): boolean {
    return commitGroupToggles[groupKey] ?? false;
  }
  function expandCommitGroup(groupKey: string) {
    commitGroupToggles = { ...commitGroupToggles, [groupKey]: true };
  }
  function isRevisionExpanded(revId: string): boolean {
    if (revId in revisionToggles) {
      return revisionToggles[revId];
    }
    return revId === latestRevisionId;
  }
  function toggleRevision(revId: string) {
    revisionToggles = {
      ...revisionToggles,
      [revId]: !isRevisionExpanded(revId),
    };
  }
  function isReviewExpanded(opId: string): boolean {
    return opId in reviewToggles ? reviewToggles[opId] : true;
  }
  function toggleReview(opId: string) {
    reviewToggles = {
      ...reviewToggles,
      [opId]: !isReviewExpanded(opId),
    };
  }

  function groupCommitsByAuthor(commits: Commit[]): Commit[][] {
    const groups: Commit[][] = [];
    for (const commit of commits) {
      const last = groups[groups.length - 1];
      if (last && last[0].author.name === commit.author.name) {
        last.push(commit);
      } else {
        groups.push([commit]);
      }
    }
    return groups;
  }

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
    _embeds: Embed[],
    _replyTo?: string,
    location?: CodeLocation,
  ) {
    if (!location) return;
    try {
      let draftId = draftReview?.id;
      if (!draftId) {
        draftId = draftReviewStorage.create(rid, patchId, revision.id);
      }
      draftReviewStorage.addComment(draftId, { body, location });
    } catch (error) {
      console.error("Creating code comment failed", error);
    } finally {
      await loadPatch();
    }
  }

  async function createCodeCommentDirect(
    body: string,
    embeds: Embed[],
    location: CodeLocation,
  ) {
    try {
      await invoke("create_patch_comment", {
        rid,
        new: {
          id: patchId,
          body,
          embeds,
          location,
          revision: revision.id,
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Creating code comment failed", error);
    } finally {
      await loadPatch();
    }
  }

  const commentToReviewId = $derived.by(() => {
    // eslint-disable-next-line svelte/prefer-svelte-reactivity
    const map = new Map<string, string>();
    for (const [reviewId, threads] of threadsByReview.entries()) {
      for (const thread of threads) {
        map.set(thread.root.id, reviewId);
        for (const reply of thread.replies) {
          map.set(reply.id, reviewId);
        }
      }
    }
    return map;
  });

  async function editCodeComment(
    commentId: string,
    body: string,
    embeds: Embed[],
  ) {
    if (draftReview?.comments.find(c => c.id === commentId)) {
      draftReviewStorage.updateComment(draftReview.id, commentId, { body });
      await loadPatch();
      return;
    }
    const reviewId = commentToReviewId.get(commentId);
    if (!reviewId) return;
    try {
      await invoke("edit_patch", {
        rid,
        cobId: patchId,
        action: {
          type: "review.comment.edit",
          review: reviewId,
          comment: commentId,
          body,
          embeds,
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Editing review comment failed", error);
    } finally {
      await loadPatch();
    }
  }

  async function deleteCodeComment(commentId: string) {
    if (draftReview?.comments.find(c => c.id === commentId)) {
      draftReviewStorage.deleteComment(draftReview.id, commentId);
      await loadPatch();
      return;
    }
    const reviewId = commentToReviewId.get(commentId);
    if (!reviewId) return;
    try {
      await invoke("edit_patch", {
        rid,
        cobId: patchId,
        action: {
          type: "review.comment.redact",
          review: reviewId,
          comment: commentId,
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Deleting review comment failed", error);
    } finally {
      await loadPatch();
    }
  }

  async function reactOnCodeComment(
    commentId: string,
    authors: Author[],
    reaction: string,
  ) {
    const reviewId = commentToReviewId.get(commentId);
    if (!reviewId) return;
    try {
      await invoke("edit_patch", {
        rid,
        cobId: patchId,
        action: {
          type: "review.comment.react",
          review: reviewId,
          comment: commentId,
          reaction,
          active: !authors.find(
            ({ did }) => publicKeyFromDid(did) === config.publicKey,
          ),
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Editing review comment reactions failed", error);
    } finally {
      await loadPatch();
    }
  }

  const publishedReviewThreads: Thread<CodeLocation>[] = $derived.by(() => {
    const list: Thread<CodeLocation>[] = [];
    for (const threads of threadsByReview.values()) {
      list.push(...threads);
    }
    return list;
  });

  // eslint-disable-next-line @typescript-eslint/no-empty-function
  const noopAsync = async () => {};

  const codeComments: CodeComments | undefined = $derived.by(() => {
    const combinedThreads = [...codeCommentThreads, ...publishedReviewThreads];
    if (hasPublishedReview) {
      if (combinedThreads.length === 0) return undefined;
      return {
        config,
        createComment: noopAsync,
        editComment: noopAsync,
        repoDelegates,
        rid,
        threads: combinedThreads,
        canReply: false,
      };
    }
    return {
      config,
      createComment: createCodeComment,
      addCodeCommentDirect: createCodeCommentDirect,
      newCommentCaption: draftReview ? "Add to review" : "Start review",
      newCommentDescription: draftReview
        ? "Save this in your draft review and publish later with a verdict."
        : "Begin a draft review. You can add more comments before publishing.",
      addCodeCommentDirectCaption: "Just comment",
      addCodeCommentDirectDescription:
        "Post this comment now, without starting or contributing to a review.",
      editComment: editCodeComment,
      deleteComment: deleteCodeComment,
      repoDelegates,
      rid,
      threads: combinedThreads,
      canReply: false,
      disableAttachments: "Publish your review to attach files",
    };
  });

  let commitsByRevision: Record<string, Commit[]> = $state({});

  $effect(() => {
    const ridLocal = rid;
    const sorted = [...revisions].sort((a, b) => a.timestamp - b.timestamp);
    void Promise.all(
      sorted.map(async (rev): Promise<[string, Commit[]]> => {
        try {
          const commits = await cachedListCommits(ridLocal, rev.base, rev.head);
          return [rev.id, commits];
        } catch (error) {
          console.error(
            `Failed to load commits for revision ${rev.id} (${rev.base}..${rev.head})`,
            error,
          );
          return [rev.id, []];
        }
      }),
    ).then(entries => {
      const next: Record<string, Commit[]> = {};
      // eslint-disable-next-line svelte/prefer-svelte-reactivity
      const seen = new Set<string>();
      sorted.forEach((rev, i) => {
        const [, commits] = entries[i];
        const novel = commits.filter(c => !seen.has(c.id));
        novel.forEach(c => seen.add(c.id));
        next[rev.id] = [...novel].reverse();
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

  const olderRevisionIds = $derived(
    new Set(revisions.filter(r => r.id !== latestRevisionId).map(r => r.id)),
  );

  const threadsByReview = $derived.by(() => {
    // eslint-disable-next-line svelte/prefer-svelte-reactivity
    const map = new Map<string, Thread<CodeLocation>[]>();
    (revision.reviews ?? []).forEach(review => {
      const reviewComments = review.comments ?? [];
      const threads = reviewComments
        .filter(c => c.location && !c.replyTo && c.id !== review.id)
        .map(root => {
          const replies = reviewComments
            .filter(c => c.replyTo === root.id)
            .sort((a, b) => a.edits[0].timestamp - b.edits[0].timestamp);
          return { root, replies } as Thread<CodeLocation>;
        });
      if (threads.length > 0) {
        map.set(review.id, threads);
      }
    });
    return map;
  });

  const activityItems: ActivityItem<ActivityData>[] = $derived.by(() => {
    const tracker: Partial<Record<Action["type"], Action>> = {};
    const items: ActivityItem<ActivityData>[] = [];
    // eslint-disable-next-line svelte/prefer-svelte-reactivity
    const reviewOpsByReviewId = new Map<
      string,
      FlattenedPatchOperation & { type: "review" }
    >();
    // eslint-disable-next-line svelte/prefer-svelte-reactivity
    const revisionOpsByRevisionId = new Map<
      string,
      FlattenedPatchOperation & { type: "revision" }
    >();
    // eslint-disable-next-line svelte/prefer-svelte-reactivity
    const redactedRevisionIds = new Set<string>();
    // eslint-disable-next-line svelte/prefer-svelte-reactivity
    const redactedReviewIds = new Set<string>();
    activity.forEach(operation => {
      operation.actions.forEach((action, actionIndex) => {
        if (skippedActivityTypes.has(action.type)) {
          if (action.type === "review.edit") {
            const reviewOp = reviewOpsByReviewId.get(action.review);
            if (reviewOp) {
              if ("verdict" in action) reviewOp.verdict = action.verdict;
              if ("summary" in action) reviewOp.summary = action.summary;
              if ("labels" in action) reviewOp.labels = action.labels;
            }
          } else if (action.type === "revision.edit") {
            const revisionOp = revisionOpsByRevisionId.get(action.revision);
            if (revisionOp) {
              revisionOp.description = action.description;
            }
          } else if (action.type === "revision.redact") {
            redactedRevisionIds.add(action.revision);
          } else if (action.type === "review.redact") {
            redactedReviewIds.add(action.review);
          }
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
        if (action.type === "label") {
          const prev =
            previous && previous.type === "label" ? previous.labels : [];
          const added = action.labels.filter(l => !prev.includes(l));
          const removed = prev.filter(l => !action.labels.includes(l));
          if (added.length === 0 && removed.length === 0) {
            tracker[action.type] = action;
            return;
          }
        }
        const op: FlattenedPatchOperation = {
          ...action,
          id: operation.id,
          author: operation.author,
          timestamp: operation.timestamp,
          previous,
        };
        tracker[action.type] = action;
        const commits =
          action.type === "revision"
            ? commitsByRevision[operation.id]
            : undefined;
        const reviewThreads =
          action.type === "review"
            ? threadsByReview.get(operation.id)
            : undefined;
        if (action.type === "review") {
          reviewOpsByReviewId.set(
            operation.id,
            op as FlattenedPatchOperation & { type: "review" },
          );
        }
        if (action.type === "revision") {
          revisionOpsByRevisionId.set(
            operation.id,
            op as FlattenedPatchOperation & { type: "revision" },
          );
        }
        items.push({
          key: `${operation.id}:${actionIndex}`,
          timestamp: operation.timestamp,
          data: { kind: "op", op, commits, reviewThreads },
        });
      });
    });

    const filtered = items.filter(item => {
      if (item.data.kind !== "op") return true;
      if (
        item.data.op.type === "revision" &&
        redactedRevisionIds.has(item.data.op.id)
      ) {
        return false;
      }
      if (
        item.data.op.type === "review" &&
        redactedReviewIds.has(item.data.op.id)
      ) {
        return false;
      }
      // The opening-draft lifecycle is folded into the "opened a draft patch"
      // label on the first revision, so drop the standalone item.
      if (
        item.data.op.type === "lifecycle" &&
        item.data.op.state.status === "draft" &&
        item.data.op.id === openingDraftOpId
      ) {
        return false;
      }
      return true;
    });
    filtered.sort((a, b) => a.timestamp - b.timestamp);
    items.length = 0;
    items.push(...filtered);

    // The patch creation is shown as a standalone "opened patch" marker; the
    // first revision itself stays in the timeline below (and folds with other
    // revisions). Synthesize the marker from the first revision operation.
    const firstRevisionOp = revisionOpsByRevisionId.get(firstRevisionId);
    const opened: ActivityItem<ActivityData>[] = firstRevisionOp
      ? [
          {
            key: `opened:${firstRevisionId}`,
            timestamp: firstRevisionOp.timestamp,
            data: {
              kind: "opened",
              op: firstRevisionOp,
              openedAsDraft: openingDraftOpId !== undefined,
            },
          },
        ]
      : [];

    if (olderRevisionIds.size < 2) {
      return [...opened, ...items];
    }

    // Replace the run of older revisions with a single toggle control. When
    // collapsed it stands in for them ("created N revisions"); when expanded it
    // sits above them and offers to collapse them back.
    const folded: ActivityItem<ActivityData>[] = [];
    let controlInserted = false;
    for (const item of items) {
      const isOlderRevisionOp =
        item.data.kind === "op" &&
        item.data.op.type === "revision" &&
        olderRevisionIds.has(item.data.op.id);
      if (!isOlderRevisionOp) {
        folded.push(item);
        continue;
      }
      if (!controlInserted) {
        folded.push({
          key: "older-revisions",
          timestamp: item.timestamp,
          data: {
            kind: "olderRevisions",
            count: olderRevisionIds.size,
            author: item.data.kind === "op" ? item.data.op.author : undefined,
            expanded: olderRevisionsExpanded,
          },
        });
        controlInserted = true;
      }
      if (olderRevisionsExpanded) {
        folded.push(item);
      }
    }
    return [...opened, ...folded];
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

  let editingDescription = $state(false);

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
      console.error("Editing revision reactions failed", error);
    } finally {
      await loadPatch();
    }
  }
</script>

<style>
  .patch-body {
    position: relative;
    margin-bottom: 2.5rem;
  }
  .patch-reactions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 1rem;
  }
  .patch-body:has(.body-actions) {
    padding-right: 4rem;
  }
  .body-actions {
    position: absolute;
    top: 0;
    right: 0;
    display: flex;
    align-items: center;
    gap: 0.125rem;
  }
  .body-action {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    padding: 0.25rem;
    border-radius: var(--border-radius-sm);
    cursor: pointer;
    color: var(--color-text-tertiary);
    opacity: 0;
    transition: opacity 150ms ease;
  }
  .patch-body:hover .body-action,
  .patch-body:focus-within .body-action,
  .body-action:focus-visible {
    opacity: 1;
  }
  .body-action:hover,
  .body-action:focus-visible {
    color: var(--color-text-primary);
    background-color: var(--color-surface-subtle);
  }
  .patch-body :global(.card-header),
  .patch-body :global(.card-body),
  .patch-body :global(.actions) {
    padding-left: 0;
    padding-right: 0;
  }
  .commit-group {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .commit-group-author {
    padding-left: 0.5rem;
    color: var(--color-text-tertiary);
  }
  .commit-group-children {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .revision-card {
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-md);
    background-color: var(--color-surface-canvas);
    margin: 0.5rem 0;
    overflow: hidden;
  }
  .revision-card-body {
    display: flex;
    flex-direction: column;
  }
  .revision-card-description {
    padding: 1rem;
  }
  .revision-card-description :global(:first-child) {
    margin-top: 0;
  }
  .revision-card-description :global(:last-child) {
    margin-bottom: 0;
  }
  .revision-card-divider {
    height: 1px;
    background-color: var(--color-border-subtle);
  }
  .revision-commits {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.5rem;
    background-color: var(--color-surface-base);
  }
  .revision-commits :global(.timeline-item .icon),
  .revision-commits :global(.older-revisions .icon) {
    background-color: var(--color-surface-base);
  }
  .revision-diff-tease .revision-diff-stats :global(.icon) {
    background-color: var(--color-surface-base);
  }
  .collapsible {
    display: grid;
    grid-template-rows: 0fr;
    transition: grid-template-rows 180ms ease-out;
    overflow: hidden;
    min-height: 0;
  }
  .collapsible.open {
    grid-template-rows: 1fr;
  }
  .collapsible-inner {
    overflow: hidden;
    min-height: 0;
  }
  .review-threads {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding-left: 1.5rem;
    margin-top: 0.5rem;
  }
  .review-threads::before {
    content: "";
    position: absolute;
    top: 0;
    bottom: 0;
    left: 1rem;
    width: 1px;
    background-color: var(--color-border-subtle);
  }
  .older-revisions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
    padding: 0.375rem 0.5rem;
    border-radius: var(--border-radius-sm);
    min-height: 2.5rem;
  }
  .older-revisions:hover,
  .older-revisions:focus-visible {
    background-color: var(--color-surface-subtle);
  }
  .older-revisions .icon {
    width: 1rem;
    display: grid;
    place-items: center;
  }
  .older-revisions .icon-stack {
    display: grid;
    width: 1rem;
    place-items: center;
  }
  .older-revisions .icon-default,
  .older-revisions .icon-hover {
    grid-area: 1 / 1;
    transition:
      opacity 150ms ease,
      transform 150ms ease;
  }
  .older-revisions .icon-hover {
    opacity: 0;
    transform: rotate(-90deg);
  }
  .older-revisions:hover .icon-default,
  .older-revisions:focus-visible .icon-default {
    opacity: 0;
    transform: rotate(90deg);
  }
  .older-revisions:hover .icon-hover,
  .older-revisions:focus-visible .icon-hover {
    opacity: 1;
    transform: rotate(0);
  }
  .summary-secondary {
    color: var(--color-text-tertiary);
  }
  .revision-diff-tease {
    padding: 1rem;
    background-color: var(--color-surface-base);
  }
  .revision-diff-loading,
  .revision-diff-error {
    color: var(--color-text-tertiary);
  }
  .revision-diff-stats {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.375rem;
    margin-bottom: 0.5rem;
    color: var(--color-text-secondary);
  }
  .file-fan {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: stretch;
  }
  .file-fan::after {
    content: "";
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
    width: 6rem;
    background: linear-gradient(
      to left,
      var(--color-surface-base),
      transparent
    );
    pointer-events: none;
    z-index: 6;
  }
  .file-fan-stack {
    --fan-overlap: 20%;
    display: flex;
    align-items: flex-start;
    overflow: hidden;
    height: 16rem;
    padding: 0.5rem 0 0;
  }
  .file-fan-card {
    position: relative;
    flex: 0 0
      calc(
        (100% + (var(--card-count, 5) - 1) * var(--fan-overlap, 20%)) /
          var(--card-count, 5)
      );
    max-width: calc(
      (100% + (var(--card-count, 5) - 1) * var(--fan-overlap, 20%)) /
        var(--card-count, 5)
    );
    height: 100%;
    margin-left: calc(var(--fan-overlap, 20%) * -1);
    background-color: var(--color-surface-canvas);
    box-shadow: 0 4px 14px rgba(0, 0, 0, 0.18);
    overflow: hidden;
  }
  .file-fan-card.first {
    margin-left: 0;
  }
  .file-fan-card-inner {
    height: 100%;
    overflow: hidden;
  }
  .file-fan-footer {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    display: flex;
    justify-content: center;
    align-items: flex-end;
    padding: 0 0 2rem;
    height: 7rem;
    z-index: 10;
    pointer-events: none;
  }
  .file-fan-footer .diff-tease-button {
    pointer-events: auto;
  }
  .file-fan-fade {
    position: absolute;
    left: 0;
    right: 0;
    top: 0;
    bottom: 0;
    background: linear-gradient(
      to bottom,
      transparent 0%,
      var(--color-surface-base) 100%
    );
    pointer-events: none;
  }
  .diff-tease-button {
    position: relative;
    z-index: 1;
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    background-color: var(--color-surface-canvas);
    color: var(--color-text-primary);
    cursor: pointer;
    box-shadow: var(--elevation-low);
  }
  .diff-tease-button:hover,
  .diff-tease-button:focus-visible {
    background-color: var(--color-surface-subtle);
  }
  .merge-card {
    display: grid;
    grid-template-columns: 1fr 1px 1fr;
    align-items: stretch;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-md);
    background-color: var(--color-surface-canvas);
    margin: 1rem 0;
    overflow: hidden;
  }
  .merge-card::before {
    content: "";
    grid-column: 2;
    background-color: var(--color-border-subtle);
  }
  .merge-card-left,
  .merge-card-right {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding: 1.25rem;
    min-width: 0;
  }
  .merge-card-left {
    grid-column: 1;
    justify-content: space-between;
  }
  .merge-card-right {
    grid-column: 3;
    gap: 0.375rem;
  }
  .merge-card-header {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
  }
  .merge-card-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 2rem;
    height: 2rem;
    border-radius: var(--border-radius-sm);
    color: var(--color-text-on-brand);
    background-color: var(--color-surface-brand-secondary);
    flex-shrink: 0;
  }
  .merge-card-text {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    min-width: 0;
  }
  .merge-card-title {
    color: var(--color-text-primary);
  }
  .merge-card-body {
    color: var(--color-text-secondary);
  }
  .merge-card-cli-label {
    color: var(--color-text-tertiary);
  }
  .merge-card-cli-code {
    margin: 0;
    padding: 0.75rem 0.875rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    background-color: var(--color-surface-subtle);
    color: var(--color-text-primary);
    font: var(--txt-body-s-regular);
    font-family:
      ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Courier New",
      monospace;
    white-space: pre-wrap;
    overflow-x: auto;
  }
  .merge-card-button {
    align-self: flex-start;
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1.25rem;
    border: 0;
    border-radius: var(--border-radius-sm);
    background-color: var(--color-feedback-success-fill);
    color: var(--color-text-on-brand);
    font: var(--txt-body-m-medium);
    cursor: pointer;
  }
  .merge-card-button:hover:not(:disabled),
  .merge-card-button:focus-visible:not(:disabled) {
    background-color: var(--color-feedback-success-fill-hover);
  }
  .merge-card-button:disabled {
    cursor: default;
    opacity: 0.6;
  }
</style>

{#if view === "description"}
  {@const body = revision.description.slice(-1)[0].body}
  {@const canEdit = roles.isDelegateOrAuthor(
    config.publicKey,
    repoDelegates.map(d => d.did),
    revision.author.did,
  )}
  {#if editingDescription}
    <div class="patch-body">
      <ExtendedTextarea
        {rid}
        body={body ?? ""}
        focus
        submitCaption="Save"
        submit={async ({ comment, embeds }) => {
          await editRevision(comment, Array.from(embeds.values()));
          editingDescription = false;
        }}
        close={() => (editingDescription = false)} />
    </div>
  {:else if body.trim() !== "" || canEdit}
    <div class="patch-body txt-body-m-regular">
      {#if body.trim() !== ""}
        <Markdown {rid} content={body} />
      {:else}
        <span style:color="var(--color-text-tertiary)">No description.</span>
      {/if}
      <div class="body-actions">
        <div class="body-action">
          <ReactionSelector
            placement="bottom-end"
            reactions={revision.reactions ?? []}
            select={async ({ authors, emoji }) => {
              try {
                await reactOnRevision(authors, emoji);
              } finally {
                closeFocused();
              }
            }} />
        </div>
        {#if canEdit}
          <button
            type="button"
            class="body-action edit-description"
            title="Edit description"
            onclick={() => (editingDescription = true)}>
            <Icon name="edit" />
          </button>
        {/if}
      </div>
    </div>
  {/if}
  {#if revision.reactions && revision.reactions.length > 0}
    <div class="patch-reactions">
      <Reactions
        handleReaction={reactOnRevision}
        currentUserNid={config.publicKey}
        reactions={revision.reactions} />
    </div>
  {/if}
{:else if view === "activity"}
  {#snippet renderActivity(data: ActivityData, opts: { hideAuthor: boolean })}
    {#if data.kind === "op"}
      {#if data.op.type === "revision"}
        {@const revId = data.op.id}
        {@const isFirst = revId === firstRevisionId}
        {@const isOlder = olderRevisionIds.has(revId)}
        {@const hasCommits = !!data.commits && data.commits.length > 0}
        {@const revBody = isFirst
          ? undefined
          : splitDescription(data.op.description).body}
        {@const hasBody = !!revBody}
        {@const toggleable = hasCommits || hasBody}
        {@const expanded = toggleable && isRevisionExpanded(revId)}
        {@const targetRev = revisions.find(r => r.id === revId)}
        {#if expanded}
          <div class="revision-card" transition:slide={{ duration: 180 }}>
            <PatchActivityItem
              op={data.op}
              {rid}
              {expanded}
              hideAuthor={opts.hideAuthor}
              bodyExternal
              onToggle={toggleable ? () => toggleRevision(revId) : undefined} />
            <div class="revision-card-body">
              {#if hasBody}
                <div class="revision-card-description txt-body-m-regular">
                  <Markdown {rid} breaks content={revBody} />
                </div>
              {/if}
              {#if hasBody && hasCommits}
                <div class="revision-card-divider"></div>
              {/if}
              {#if hasCommits && data.commits && data.commits.length > 0}
                <div class="revision-commits">
                  {#each groupCommitsByAuthor(data.commits) as group (group[0].id)}
                    {#if group.length > 1}
                      {@const groupKey = group[0].id}
                      {@const groupExpanded = isCommitGroupExpanded(groupKey)}
                      {@const collapsed =
                        !groupExpanded &&
                        group.length > COMMIT_COLLAPSE_THRESHOLD}
                      {@const visibleCommits = collapsed
                        ? group.slice(0, MAX_COMMITS_VISIBLE)
                        : group}
                      {@const hiddenCount = collapsed
                        ? group.length - MAX_COMMITS_VISIBLE
                        : 0}
                      <div class="commit-group">
                        <div class="commit-group-author txt-body-m-regular">
                          {group[0].author.name} &lt;{group[0].author.email}&gt;
                          committed
                        </div>
                        <div class="commit-group-children">
                          {#each visibleCommits as commit (commit.id)}
                            <CommitActivityItem {commit} hideAuthor />
                          {/each}
                          {#if collapsed}
                            <!-- svelte-ignore a11y_click_events_have_key_events -->
                            <div
                              class="older-revisions txt-body-m-regular"
                              role="button"
                              tabindex="0"
                              transition:slide={{ duration: 180 }}
                              onclick={() => expandCommitGroup(groupKey)}>
                              <div class="icon">
                                <span class="icon-stack">
                                  <span class="icon-default">
                                    <Icon name="commit" />
                                  </span>
                                  <span class="icon-hover">
                                    <Icon name="expand-vertical" />
                                  </span>
                                </span>
                              </div>
                              <span class="summary-secondary">
                                {hiddenCount} more
                                {hiddenCount === 1 ? "commit" : "commits"}
                              </span>
                            </div>
                          {/if}
                        </div>
                      </div>
                    {:else}
                      <CommitActivityItem commit={group[0]} />
                    {/if}
                  {/each}
                </div>
              {/if}
              {#if targetRev && targetRev.base !== targetRev.head}
                {#if hasBody || hasCommits}
                  <div class="revision-card-divider"></div>
                {/if}
                <div class="revision-diff-tease">
                  {#await cachedGetDiff( rid, { base: targetRev.base, head: targetRev.head, unified: 3, highlight: true }, )}
                    <div class="revision-diff-loading txt-body-m-regular">
                      Loading diff…
                    </div>
                  {:then diff}
                    {@const previewFiles = diff.files
                      .filter(f => !isIgnoredFile(f))
                      .slice(0, 5)}
                    <div class="revision-diff-stats txt-body-m-regular">
                      <Icon name="diff" />
                      <span>
                        {diff.stats.filesChanged}
                        {pluralize("file", diff.stats.filesChanged)} modified with
                      </span>
                      <span style:color="var(--color-feedback-success-text)">
                        {diff.stats.insertions}
                        {pluralize("insertion", diff.stats.insertions)}
                      </span>
                      <span>and</span>
                      <span style:color="var(--color-feedback-error-text)">
                        {diff.stats.deletions}
                        {pluralize("deletion", diff.stats.deletions)}
                      </span>
                    </div>
                    <div class="file-fan">
                      <div
                        class="file-fan-stack"
                        style:--card-count={previewFiles.length}>
                        {#each previewFiles as file, i (i)}
                          <div
                            class="file-fan-card"
                            style:z-index={i + 1}
                            class:first={i === 0}>
                            <div class="file-fan-card-inner">
                              <FileDiff
                                {file}
                                head={targetRev.head}
                                expanded
                                expandable={false} />
                            </div>
                          </div>
                        {/each}
                      </div>
                      <div class="file-fan-footer">
                        <div class="file-fan-fade"></div>
                        <button
                          type="button"
                          class="diff-tease-button txt-body-m-medium"
                          disabled={!onViewChanges}
                          onclick={() => onViewChanges?.(revId)}>
                          View all revision changes
                          <Icon name="arrow-right" />
                        </button>
                      </div>
                    </div>
                  {:catch error}
                    <div class="revision-diff-error txt-body-m-regular">
                      Failed to load diff: {error?.message ?? error}
                    </div>
                  {/await}
                </div>
              {/if}
            </div>
          </div>
        {:else if isOlder}
          <div
            class="older-revision-entry"
            transition:slide={{ duration: 180 }}>
            <PatchActivityItem
              op={data.op}
              {rid}
              {expanded}
              hideAuthor={opts.hideAuthor}
              onToggle={toggleable ? () => toggleRevision(revId) : undefined} />
          </div>
        {:else}
          <PatchActivityItem
            op={data.op}
            {rid}
            {expanded}
            hideAuthor={opts.hideAuthor}
            onToggle={toggleable ? () => toggleRevision(revId) : undefined} />
        {/if}
        {#if revId === latestRevisionId}
          {@render mergeCard()}
        {/if}
      {:else if data.op.type === "review"}
        {@const opId = data.op.id}
        {@const threads = data.reviewThreads ?? []}
        {@const hasThreads = threads.length > 0}
        {@const reviewRecord = (revision.reviews ?? []).find(
          r => r.id === opId,
        )}
        {@const hasReviewComments = (reviewRecord?.comments?.length ?? 0) > 0}
        {@const toggleable = hasThreads}
        {@const expanded = toggleable ? isReviewExpanded(opId) : true}
        <PatchActivityItem
          op={data.op}
          expanded={toggleable ? expanded : undefined}
          onToggle={toggleable ? () => toggleReview(opId) : undefined}
          hideAuthor={opts.hideAuthor}
          onViewFullReview={hasReviewComments
            ? () =>
                void push({
                  resource: "repo.patch",
                  rid,
                  patch: patchId,
                  status: undefined,
                  reviewId: opId,
                })
            : undefined} />
        {#if hasThreads}
          <div class="collapsible" class:open={expanded}>
            <div class="collapsible-inner">
              <div class="review-threads">
                {#each threads as thread (thread.root.id)}
                  <ReviewCodeThread
                    {rid}
                    base={revision.base}
                    head={revision.head}
                    {thread}
                    {config}
                    {repoDelegates}
                    editComment={editCodeComment}
                    deleteComment={deleteCodeComment}
                    reactOnComment={reactOnCodeComment} />
                {/each}
              </div>
            </div>
          </div>
        {/if}
      {:else}
        <PatchActivityItem
          op={data.op}
          hideAuthor={opts.hideAuthor}
          targetBranch={data.op.type === "merge" ? targetBranch : undefined} />
      {/if}
    {:else if data.kind === "opened"}
      <PatchActivityItem
        op={data.op}
        firstRevision
        openedAsDraft={data.openedAsDraft}
        hideAuthor={opts.hideAuthor} />
    {:else if data.kind === "olderRevisions"}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div
        class="older-revisions txt-body-m-regular"
        role="button"
        tabindex="0"
        transition:slide={{ duration: 180 }}
        onclick={() => (olderRevisionsExpanded = !olderRevisionsExpanded)}>
        <div class="icon">
          <span class="icon-stack">
            <span class="icon-default">
              <Icon name={data.expanded ? "collapse-vertical" : "revision"} />
            </span>
            <span class="icon-hover">
              <Icon
                name={data.expanded
                  ? "collapse-vertical"
                  : "expand-vertical"} />
            </span>
          </span>
        </div>
        <span class="summary-secondary">
          {#if data.expanded}
            collapse {data.count}
            {data.count === 1 ? "revision" : "revisions"}
          {:else}
            created {data.count}
            {data.count === 1 ? "revision" : "revisions"}
          {/if}
        </span>
      </div>
    {/if}
  {/snippet}

  {#snippet mergeCard()}
    {#if onMerge}
      {@const branch =
        repo.payloads["xyz.radicle.project"]?.data.defaultBranch ?? "main"}
      <div
        class="merge-card"
        class:disabled={mergeDisabledReason !== undefined}>
        <div class="merge-card-left">
          <div class="merge-card-header">
            <div class="merge-card-icon">
              <Icon name="patch-merged" />
            </div>
            <div class="merge-card-text">
              <div class="merge-card-title txt-body-m-medium">
                Ready to merge
              </div>
              <div class="merge-card-body txt-body-m-regular">
                Merge this revision into <b>{branch}</b>
                .
              </div>
            </div>
          </div>
          <button
            type="button"
            class="merge-card-button txt-body-m-medium"
            disabled={mergeDisabledReason !== undefined}
            onclick={() => void onMerge?.()}
            title={mergeDisabledReason ?? "Record this revision as merged"}>
            <Icon name="patch-merged" />
            Merge revision
          </button>
        </div>
        <div class="merge-card-right">
          <div class="merge-card-cli-label txt-body-m-regular">
            Or merge from the command line:
          </div>
          <pre class="merge-card-cli-code">$ rad patch checkout {patchId.slice(
              0,
              7,
            )}
$ git checkout {branch}
$ git merge --no-ff patch/{patchId.slice(0, 7)}
$ git push rad {branch}</pre>
        </div>
      </div>
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
    {renderActivity}
    authorOf={data =>
      data.kind === "op" || data.kind === "opened"
        ? data.op.author
        : data.author} />
{:else}
  <Changes
    {rid}
    {patchId}
    {revision}
    {codeComments}
    {draftReviewId}
    bind:filesExpanded />
{/if}
