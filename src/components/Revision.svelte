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
  import { cachedListCommits, invoke } from "@app/lib/invoke";
  import * as roles from "@app/lib/roles";
  import { push } from "@app/lib/router";
  import { didFromPublicKey, publicKeyFromDid } from "@app/lib/utils";

  import { announce } from "@app/components/AnnounceSwitch.svelte";
  import Changes from "@app/components/Changes.svelte";
  import ExtendedTextarea from "@app/components/ExtendedTextarea.svelte";
  import Markdown from "@app/components/Markdown.svelte";
  import CommitActivityItem from "@app/components/CommitActivityItem.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Discussion, {
    type ActivityItem,
  } from "@app/components/Discussion.svelte";
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
    | { kind: "olderRevisions"; count: number; author?: Author };

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
  }

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
  }: Props = $props();
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
  const targetBranch = $derived(
    repo.payloads["xyz.radicle.project"]?.data.defaultBranch,
  );
  let revisionToggles: Record<string, boolean> = $state({});
  let reviewToggles: Record<string, boolean> = $state({});
  let commitGroupToggles: Record<string, boolean> = $state({});
  let olderRevisionsExpanded = $state(false);
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
    return revId === latestRevisionId || revId === firstRevisionId;
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
      draftReviewStorage.updateComment(draftReview.id, commentId, {
        body,
        embeds,
      });
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
    new Set(
      revisions
        .filter(r => r.id !== firstRevisionId && r.id !== latestRevisionId)
        .map(r => r.id),
    ),
  );

  const threadsByReview = $derived.by(() => {
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
    const reviewOpsByReviewId = new Map<
      string,
      FlattenedPatchOperation & { type: "review" }
    >();
    const revisionOpsByRevisionId = new Map<
      string,
      FlattenedPatchOperation & { type: "revision" }
    >();
    const redactedRevisionIds = new Set<string>();
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
      return true;
    });
    filtered.sort((a, b) => a.timestamp - b.timestamp);
    items.length = 0;
    items.push(...filtered);

    if (olderRevisionsExpanded || olderRevisionIds.size < 2) {
      return items;
    }

    const folded: ActivityItem<ActivityData>[] = [];
    let placeholderInserted = false;
    let placeholderAuthor: Author | undefined;
    for (const item of items) {
      const isOlderRevisionOp =
        item.data.kind === "op" &&
        item.data.op.type === "revision" &&
        olderRevisionIds.has(item.data.op.id);
      if (!isOlderRevisionOp) {
        folded.push(item);
        continue;
      }
      if (!placeholderInserted) {
        placeholderAuthor =
          item.data.kind === "op" ? item.data.op.author : undefined;
        folded.push({
          key: "older-revisions",
          timestamp: item.timestamp,
          data: {
            kind: "olderRevisions",
            count: olderRevisionIds.size,
            author: placeholderAuthor,
          },
        });
        placeholderInserted = true;
      }
    }
    return folded;
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
    margin-bottom: 1rem;
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
    color: var(--color-text-tertiary);
  }
  .commit-group-children {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .revision-commits {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding-left: 1.25rem;
  }
  .revision-commits.has-header {
    margin-top: 0.5rem;
  }
  .revision-commits::before {
    content: "";
    position: absolute;
    top: 0;
    bottom: 0;
    left: 0.5rem;
    width: 1px;
    background-color: var(--color-border-subtle);
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
    padding-left: 1.25rem;
    margin-top: 0.5rem;
  }
  .review-threads::before {
    content: "";
    position: absolute;
    top: 0;
    bottom: 0;
    left: 0.5rem;
    width: 1px;
    background-color: var(--color-border-subtle);
  }
  .older-revisions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
    padding: 0.375rem 0.5rem;
    margin: 0 -0.5rem;
    border-radius: var(--border-radius-sm);
    min-height: 2.5rem;
  }
  .older-revisions:hover,
  .older-revisions:focus-visible {
    background-color: var(--color-surface-subtle);
  }
  .older-revisions .icon {
    padding-top: 0.1875rem;
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
        {@const hasBody =
          !isFirst && !!splitDescription(data.op.description).body}
        {@const toggleable = hasCommits || hasBody}
        {@const expanded = toggleable && isRevisionExpanded(revId)}
        {#if isOlder}
          <div
            class="older-revision-entry"
            transition:slide={{ duration: 180 }}>
            <PatchActivityItem
              op={data.op}
              {rid}
              {expanded}
              hideAuthor={opts.hideAuthor}
              firstRevision={isFirst}
              onToggle={toggleable
                ? () => toggleRevision(revId)
                : undefined} />
          </div>
        {:else}
          <PatchActivityItem
            op={data.op}
            {rid}
            {expanded}
            hideAuthor={opts.hideAuthor}
            firstRevision={isFirst}
            onToggle={toggleable
              ? () => toggleRevision(revId)
              : undefined} />
        {/if}
        {#if expanded && data.commits && data.commits.length > 0}
          <div
            class="revision-commits has-header"
            transition:slide={{ duration: 180 }}>
            {#each groupCommitsByAuthor(data.commits) as group (group[0].id)}
              {#if group.length > 1}
                {@const groupKey = group[0].id}
                {@const groupExpanded = isCommitGroupExpanded(groupKey)}
                {@const collapsed =
                  !groupExpanded && group.length > COMMIT_COLLAPSE_THRESHOLD}
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
                      <CommitActivityItem
                        {commit}
                        {rid}
                        {codeComments}
                        {draftReviewId}
                        hideAuthor />
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
                            <span class="icon-default"
                              ><Icon name="commit" /></span>
                            <span class="icon-hover"
                              ><Icon name="expand-vertical" /></span>
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
                <CommitActivityItem
                  commit={group[0]}
                  {rid}
                  {codeComments}
                  {draftReviewId} />
              {/if}
            {/each}
          </div>
        {/if}
      {:else if data.op.type === "review"}
        {@const opId = data.op.id}
        {@const threads = data.reviewThreads ?? []}
        {@const hasThreads = threads.length > 0}
        {@const reviewRecord = (revision.reviews ?? []).find(
          r => r.id === opId,
        )}
        {@const hasReviewComments =
          (reviewRecord?.comments?.length ?? 0) > 0}
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
    {:else if data.kind === "olderRevisions"}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
      <div
        class="older-revisions txt-body-m-regular"
        role="button"
        tabindex="0"
        transition:slide={{ duration: 180 }}
        onclick={() => (olderRevisionsExpanded = true)}>
        <div class="icon">
          <span class="icon-stack">
            <span class="icon-default"><Icon name="revision" /></span>
            <span class="icon-hover"><Icon name="expand-vertical" /></span>
          </span>
        </div>
        <span class="summary-secondary">
          created {data.count}
          {data.count === 1 ? "revision" : "revisions"}
        </span>
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
    authorOf={data => (data.kind === "op" ? data.op.author : data.author)} />
{:else}
  <Changes
    {rid}
    {patchId}
    {revision}
    {codeComments}
    {draftReviewId}
    bind:filesExpanded />
{/if}
