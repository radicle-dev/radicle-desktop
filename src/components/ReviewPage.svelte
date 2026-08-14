<script lang="ts">
  import type { CommentOrigin } from "@app/components/Comment.svelte";
  import type { PatchView } from "@app/views/repo/router";
  import type { Author } from "@bindings/cob/Author";
  import type { Patch } from "@bindings/cob/patch/Patch";
  import type { Review } from "@bindings/cob/patch/Review";
  import type { Revision } from "@bindings/cob/patch/Revision";
  import type { Verdict } from "@bindings/cob/patch/Verdict";
  import type { CodeLocation } from "@bindings/cob/thread/CodeLocation";
  import type { Comment } from "@bindings/cob/thread/Comment";
  import type { Embed } from "@bindings/cob/thread/Embed";
  import type { Thread } from "@bindings/cob/thread/Thread";
  import type { Config } from "@bindings/config/Config";
  import type { FileDiff } from "@bindings/diff/FileDiff";

  import { untrack } from "svelte";

  import type { CommentOwner } from "@app/lib/codeCommentActions";
  import { commentActions } from "@app/lib/codeCommentActions";
  import type { CodeComments } from "@app/lib/codeComments";
  import { diffOptions } from "@app/lib/diffOptions.svelte";
  import { fileMetaOf, fullFileLoader } from "@app/lib/diffText";
  import { nodeRunning } from "@app/lib/events";
  import {
    cachedGetDiff,
    cachedGetDiffText,
    getDiffText,
    invoke,
  } from "@app/lib/invoke";
  import { commentCountsByPath } from "@app/lib/pierreComments";
  import * as roles from "@app/lib/roles";
  import { push } from "@app/lib/router";
  import {
    absoluteTimestamp,
    authorForNodeId,
    didFromPublicKey,
    formatTimestamp,
    publicKeyFromDid,
    revisionPosition,
    verdictAction,
    verdictBadge,
    verdictIcon,
  } from "@app/lib/utils";

  import { announce } from "@app/components/AnnounceSwitch.svelte";
  import Button from "@app/components/Button.svelte";
  import Discussion from "@app/components/Discussion.svelte";
  import DropdownList from "@app/components/DropdownList.svelte";
  import DropdownListItem from "@app/components/DropdownListItem.svelte";
  import ExtendedTextarea from "@app/components/ExtendedTextarea.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import Markdown from "@app/components/Markdown.svelte";
  import NodeId from "@app/components/NodeId.svelte";
  import PierreDiff from "@app/components/PierreDiff.svelte";
  import Popover, { closeFocused } from "@app/components/Popover.svelte";

  interface Props {
    config: Config;
    loadPatch: () => Promise<void>;
    patch: Patch;
    repoDelegates: Author[];
    review: Review;
    revisions: Revision[];
    rid: string;
    status: Patch["state"]["status"] | undefined;
    // The tab this review was opened from, so leaving it returns there.
    fromView?: PatchView;
  }

  const {
    config,
    loadPatch,
    patch,
    repoDelegates,
    review,
    revisions,
    rid,
    status,
    fromView,
  }: Props = $props();

  const isOwnPublishedReview = $derived(
    review.author.did === didFromPublicKey(config.publicKey),
  );

  let verdictPickerExpanded = $state(false);
  let savingVerdict = $state(false);
  let editingSummary = $state(false);
  let savingSummary = $state(false);
  let deleteConfirmExpanded = $state(false);
  let deleting = $state(false);

  const verdictOptions: { value: Verdict | undefined; label: string }[] = (
    [undefined, "accept", "reject"] as const
  ).map(value => ({ value, label: verdictAction(value) }));

  async function createCodeComment(
    body: string,
    embeds: Embed[],
    replyTo?: string,
    location?: CodeLocation,
  ) {
    // Commenting from inside a review joins that review, whoever authored it,
    // so the comment is resolvable and travels with the verdict. Attribution
    // stays per-comment, and the protocol allows anyone to comment on a review.
    if (!replyTo) {
      if (!location) return;
      try {
        await invoke("edit_patch", {
          rid,
          cobId: patch.id,
          action: {
            type: "review.comment",
            review: review.id,
            body,
            location,
            embeds,
          },
          opts: { announce: $nodeRunning && $announce },
        });
      } catch (error) {
        console.error("Adding comment to review failed", error);
      } finally {
        await loadPatch();
      }
      return;
    }
    // Replies follow their parent: into the review, or onto the revision when
    // replying to a standalone comment. Without a revision there is nothing to
    // attach the latter to, so don't send an action with an empty target.
    const toReview = reviewCommentIds.has(replyTo);
    if (!toReview && !reviewedRevision) return;
    try {
      await invoke("edit_patch", {
        rid,
        cobId: patch.id,
        action: toReview
          ? {
              type: "review.comment",
              review: review.id,
              body,
              replyTo,
              embeds,
            }
          : {
              type: "revision.comment",
              revision: reviewedRevision?.id,
              body,
              replyTo,
              embeds,
            },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Replying to comment failed", error);
    } finally {
      await loadPatch();
    }
  }

  async function saveSummary(newSummary: string) {
    try {
      savingSummary = true;
      await invoke("edit_patch", {
        rid,
        cobId: patch.id,
        action: {
          type: "review.edit",
          review: review.id,
          summary: newSummary,
          verdict: review.verdict,
          labels: review.labels,
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Editing review summary failed", error);
    } finally {
      savingSummary = false;
      editingSummary = false;
      await loadPatch();
    }
  }

  async function deleteReview() {
    try {
      deleting = true;
      closeFocused();
      await invoke("edit_patch", {
        rid,
        cobId: patch.id,
        action: { type: "review.redact", review: review.id },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Deleting review failed", error);
      deleting = false;
      return;
    }
    deleting = false;
    backToPatch();
    await loadPatch();
  }

  async function setVerdict(verdict: Verdict | undefined) {
    if (verdict === review.verdict) {
      closeFocused();
      return;
    }
    try {
      savingVerdict = true;
      closeFocused();
      await invoke("edit_patch", {
        rid,
        cobId: patch.id,
        action: {
          type: "review.edit",
          review: review.id,
          summary: review.summary ?? "",
          verdict,
          labels: review.labels,
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Editing review verdict failed", error);
    } finally {
      savingVerdict = false;
      await loadPatch();
    }
  }

  // Comments on the review itself rather than on a line. The summary is a
  // field, not the root of this thread, so a comment replying to the review id
  // starts a root thread of its own.
  const commentThreads: Thread[] = $derived.by(() => {
    const comments = (review.comments ?? []) as Comment<CodeLocation>[];
    return comments
      .filter(c => (!c.location && !c.replyTo) || c.replyTo === review.id)
      .map(root => ({
        root,
        replies: comments
          .filter(c => c.replyTo === root.id)
          .sort((a, b) => a.edits[0].timestamp - b.edits[0].timestamp),
      })) as unknown as Thread[];
  });

  async function createDiscussionComment(
    body: string,
    embeds: Embed[],
    replyTo?: string,
  ) {
    try {
      await invoke("edit_patch", {
        rid,
        cobId: patch.id,
        action: {
          type: "review.comment",
          review: review.id,
          body,
          replyTo,
          embeds,
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Commenting on review failed", error);
    } finally {
      await loadPatch();
    }
  }

  type FileGroup = {
    path: string;
    threads: Thread<CodeLocation>[];
  };

  const reviewedRevision: Revision | undefined = $derived(
    revisions.find(r => r.reviews?.some(rev => rev.id === review.id)),
  );

  // 1-based position of the reviewed revision, so the page can say which
  // revision (of how many) this review is of.
  const reviewedRevisionNumber = $derived(
    reviewedRevision
      ? revisionPosition(revisions, reviewedRevision.id)
      : undefined,
  );

  // Ids of comments that belong to the review itself (as opposed to standalone
  // code comments left directly on the revision), so edits/replies target the
  // right action.
  const reviewCommentIds = $derived(
    new Set((review.comments as Comment<CodeLocation>[]).map(c => c.id)),
  );

  // Threads here mix this review's own comments with standalone comments on
  // the reviewed revision, so each mutation resolves its target first.
  function ownerOf(commentId: string): CommentOwner | undefined {
    if (reviewCommentIds.has(commentId)) {
      return { kind: "review", reviewId: review.id };
    }
    if (reviewedRevision) {
      return { kind: "revision", revisionId: reviewedRevision.id };
    }
    return undefined;
  }

  // The protocol lets the comment author, the review author or the revision
  // author resolve a review comment; delegates may do anything. Standalone
  // revision comments have no resolve action at all.
  function canResolveComment(commentId: string): boolean {
    if (ownerOf(commentId)?.kind !== "review") return false;
    if (
      roles.isDelegate(
        config.publicKey,
        repoDelegates.map(d => d.did),
      )
    ) {
      return true;
    }
    const comment = (review.comments ?? []).find(c => c.id === commentId);
    return [
      comment?.author.did,
      review.author.did,
      reviewedRevision?.author.did,
    ].some(
      did => did !== undefined && publicKeyFromDid(did) === config.publicKey,
    );
  }

  const codeActions = $derived(
    commentActions({
      rid,
      patchId: patch.id,
      publicKey: config.publicKey,
      announce: $nodeRunning && $announce,
      ownerOf,
      reload: loadPatch,
    }),
  );

  const fileGroups: FileGroup[] = $derived.by(() => {
    // eslint-disable-next-line svelte/prefer-svelte-reactivity
    const groups = new Map<string, Thread<CodeLocation>[]>();
    const addThreads = (
      comments: Comment<CodeLocation>[],
      excludeId: string,
    ) => {
      const roots = comments.filter(
        c => c.location && !c.replyTo && c.id !== excludeId,
      );
      for (const root of roots) {
        const replies = comments
          .filter(c => c.replyTo === root.id)
          .sort((a, b) => a.edits[0].timestamp - b.edits[0].timestamp);
        const thread = { root, replies } as Thread<CodeLocation>;
        const path = root.location!.path;
        const list = groups.get(path) ?? [];
        list.push(thread);
        groups.set(path, list);
      }
    };
    addThreads(review.comments as Comment<CodeLocation>[], review.id);
    // Standalone code comments left directly on the reviewed revision (not part
    // of the review itself) also belong on this page.
    if (reviewedRevision?.discussion) {
      addThreads(reviewedRevision.discussion, reviewedRevision.id);
    }
    return [...groups.entries()].map(([path, threads]) => ({ path, threads }));
  });

  // Standalone comments render next to the review's own and look identical,
  // but nothing can resolve them and their edits target the revision, so they
  // say where they live.
  const threadOrigins: Record<string, CommentOrigin> = $derived.by(() => {
    const origins: Record<string, CommentOrigin> = {};
    for (const group of fileGroups) {
      for (const thread of group.threads) {
        if (reviewCommentIds.has(thread.root.id)) continue;
        origins[thread.root.id] = {
          text: "on the revision",
          title: "Left directly on the revision, not part of this review",
        };
      }
    }
    return origins;
  });

  // File paths that carry review comments; those files start expanded, the
  // rest of the changed files render collapsed.

  const commentedPaths = $derived(new Set(fileGroups.map(g => g.path)));

  const reviewCodeComments: CodeComments = $derived({
    config,
    repoDelegates,
    rid,
    threads: fileGroups.flatMap(g => g.threads),
    threadOrigins,
    canReply: true,
    hideThreadFileHeader: true,
    createComment: createCodeComment,
    editComment: codeActions.editComment,
    deleteComment: codeActions.deleteComment,
    changeCommentStatus: codeActions.changeCommentStatus,
    canResolveComment,
    reactOnComment: codeActions.reactOnComment,
  });

  // Both halves of the reviewed revision's diff, loaded together and published
  // in one go: Pierre renders from the patch text, while the structured diff
  // supplies the per-file status and the binary/empty marks it cannot derive
  // from a hunk-less file.
  //
  // `key` doubles as Pierre's `cacheKey` prefix, so it has to be published in
  // the same update as the text it belongs to: keying the shared highlight cache
  // by file path alone collides across diffs and renders files blank.
  let loadedDiff = $state.raw<
    { key: string; text: string; files: FileDiff[] } | undefined
  >();
  $effect(() => {
    const rev = reviewedRevision;
    if (!rev) return;
    const ridLocal = rid;
    const key = `${rev.base}-${rev.head}`;
    if (untrack(() => loadedDiff?.key) === key) return;
    let cancelled = false;
    void Promise.all([
      cachedGetDiffText(ridLocal, rev.base, rev.head, 3),
      cachedGetDiff(ridLocal, {
        base: rev.base,
        head: rev.head,
      }),
    ])
      .then(([text, diff]) => {
        if (cancelled) return;
        loadedDiff = { key, text, files: diff.files };
      })
      .catch((error: unknown) => {
        if (cancelled) return;
        console.error(
          `ReviewPage: failed to load the diff for ${rev.base}..${rev.head}`,
          error,
        );
      });
    return () => {
      cancelled = true;
    };
  });

  const diffFiles = $derived(loadedDiff?.files ?? []);
  const fileMeta = $derived(fileMetaOf(diffFiles));
  const loadFullFile = $derived(
    fullFileLoader(
      rid,
      reviewedRevision?.base,
      reviewedRevision?.head ?? "",
      () => diffFiles,
    ),
  );

  // Only the files this review says something about are worth opening; the rest
  // are context, and so are the lockfiles either way.
  const collapsedPaths = $derived.by(() => {
    // eslint-disable-next-line svelte/prefer-svelte-reactivity -- rebuilt fresh each derivation
    const paths = new Set(fileMeta.ignored);
    for (const path of fileMeta.statuses.keys()) {
      if (!commentedPaths.has(path)) {
        paths.add(path);
      }
    }
    return paths;
  });

  const commentCounts = $derived(
    commentCountsByPath(
      reviewCodeComments.threads,
      commentId => reviewCodeComments.canResolveComment?.(commentId) ?? true,
    ),
  );

  const verdict = $derived(review.verdict);
  const timestamp = $derived(review.timestamp);
  const summary = $derived(review.summary?.trim() ?? "");
  const backCaption = $derived(
    fromView === "changes" ? "Back to changes" : "Back to activity",
  );

  function backToPatch() {
    void push({
      resource: "repo.patch",
      rid,
      patch: patch.id,
      status,
      reviewId: undefined,
      view: fromView === "changes" ? "changes" : undefined,
    });
  }
</script>

<style>
  .header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
    margin-bottom: 0.75rem;
  }
  .back {
    background-color: var(--color-surface-canvas);
    border: 1px solid var(--color-border-subtle);
    padding: 0.375rem 0.625rem;
    border-radius: var(--border-radius-sm);
    cursor: pointer;
    color: var(--color-text-secondary);
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    font: var(--txt-body-m-regular);
  }
  .back:hover,
  .back:focus-visible {
    color: var(--color-text-primary);
    background-color: var(--color-surface-subtle);
  }
  .meta {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin-bottom: 1.5rem;
    color: var(--color-text-secondary);
    font: var(--txt-body-m-regular);
  }
  .meta :global(*) {
    font: inherit;
  }
  .verdict-chip {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.125rem 0.5rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    background-color: var(--color-surface-canvas);
    /* Inherit the meta text colour; without this the <button> variant falls
       back to the UA `ButtonText` system colour, which is unreadable on the
       dark surface. */
    color: inherit;
    font: inherit;
  }
  .verdict-chip.accept {
    background-color: var(--color-feedback-success-bg);
    border-color: transparent;
    color: var(--color-feedback-success-text);
  }
  .verdict-chip.reject {
    background-color: var(--color-feedback-error-bg);
    border-color: transparent;
    color: var(--color-feedback-error-text);
  }
  .summary {
    position: relative;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-md);
    padding: 0.75rem 1rem;
    margin-bottom: 1.5rem;
    font: var(--txt-body-m-regular);
  }
  .summary-empty {
    color: var(--color-text-tertiary);
  }
  .summary-edit {
    position: absolute;
    top: 0.375rem;
    right: 0.375rem;
    opacity: 0;
    transition: opacity 150ms ease;
  }
  .summary:hover .summary-edit,
  .summary:focus-within .summary-edit,
  .summary-edit:focus-visible {
    opacity: 1;
  }
  .summary-editor {
    margin-bottom: 1.5rem;
  }
  .action-icon {
    background: none;
    border: 1px solid transparent;
    cursor: pointer;
    color: var(--color-text-tertiary);
    padding: 0.25rem;
    border-radius: var(--border-radius-sm);
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }
  .action-icon:hover:not(:disabled),
  .action-icon:focus-visible {
    color: var(--color-text-primary);
    background-color: var(--color-surface-subtle);
  }
  .action-icon:disabled {
    cursor: progress;
    opacity: 0.6;
  }
  .delete-confirm {
    padding: 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    background-color: var(--color-surface-canvas);
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    max-width: 20rem;
  }
  .delete-confirm-message {
    font: var(--txt-body-m-regular);
    color: var(--color-text-primary);
  }
  .delete-confirm-actions {
    display: flex;
    gap: 0.5rem;
    justify-content: flex-end;
  }
  .delete-confirm-button {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    background-color: var(--color-feedback-error-bg);
    color: var(--color-feedback-error-text);
    border: 1px solid var(--color-feedback-error-text);
    border-radius: var(--border-radius-sm);
    padding: 0.25rem 0.75rem;
    cursor: pointer;
    font: var(--txt-body-m-medium);
  }
  .delete-confirm-button:hover:not(:disabled),
  .delete-confirm-button:focus-visible {
    background-color: var(--color-feedback-error-text);
    color: var(--color-surface-canvas);
  }
  .delete-confirm-button:disabled {
    cursor: progress;
    opacity: 0.6;
  }
  /* The chrome is full width inside the diff's scroll content, so it carries
     its own horizontal padding — the same as the patch view's own header. */
  .chrome {
    display: flex;
    flex-direction: column;
    padding: 1rem 1rem 0;
  }
  .timestamp {
    color: var(--color-text-quaternary);
  }
  .verdict-toggle {
    cursor: pointer;
    font: inherit;
  }
  .verdict-toggle:not(.accept):not(.reject):hover,
  .verdict-toggle:not(.accept):not(.reject):focus-visible {
    background-color: var(--color-surface-subtle);
  }
  .verdict-toggle.accept:hover,
  .verdict-toggle.accept:focus-visible,
  .verdict-toggle.reject:hover,
  .verdict-toggle.reject:focus-visible {
    filter: brightness(0.97);
  }
  .verdict-toggle:disabled {
    cursor: progress;
    opacity: 0.6;
  }
  .verdict-accept {
    color: var(--color-feedback-success-text);
  }
  .verdict-reject {
    color: var(--color-feedback-error-text);
  }
</style>

{#snippet chrome()}
  <div class="chrome">
    <div class="header">
      <button
        type="button"
        class="back"
        onclick={backToPatch}
        title={backCaption}>
        <Icon name="arrow-left" />
        {backCaption}
      </button>
    </div>

    <div class="meta">
      <NodeId {...authorForNodeId(review.author)} />
      {#if isOwnPublishedReview}
        {@const verdictLabel = verdictBadge(verdict).label}
        <Popover
          popoverPadding="0"
          placement="bottom-start"
          bind:expanded={verdictPickerExpanded}>
          {#snippet toggle(onclick)}
            <button
              type="button"
              class="verdict-chip verdict-toggle"
              class:accept={verdict === "accept"}
              class:reject={verdict === "reject"}
              aria-haspopup="menu"
              aria-expanded={verdictPickerExpanded}
              disabled={savingVerdict}
              {onclick}>
              <Icon name={verdictIcon(verdict)} />
              {verdictLabel}
              <Icon
                name={verdictPickerExpanded ? "chevron-up" : "chevron-down"} />
            </button>
          {/snippet}
          {#snippet popover()}
            <div
              style:border="1px solid var(--color-border-subtle)"
              style:border-radius="var(--border-radius-sm)"
              style:background-color="var(--color-surface-canvas)"
              style:min-width="10rem">
              <DropdownList items={verdictOptions}>
                {#snippet item(option)}
                  {@const disabled =
                    option.value === undefined && summary === ""}
                  <DropdownListItem
                    selected={verdict === option.value}
                    {disabled}
                    title={disabled
                      ? "Add a summary before switching to Comment"
                      : undefined}
                    styleGap="0.5rem"
                    onclick={() => setVerdict(option.value)}>
                    <span
                      class:verdict-accept={option.value === "accept"}
                      class:verdict-reject={option.value === "reject"}>
                      <Icon name={verdictIcon(option.value)} />
                    </span>
                    {option.label}
                  </DropdownListItem>
                {/snippet}
              </DropdownList>
            </div>
          {/snippet}
        </Popover>
      {:else}
        <span
          class="verdict-chip"
          class:accept={verdict === "accept"}
          class:reject={verdict === "reject"}>
          <Icon name={verdictIcon(verdict)} />
          {verdict === "accept"
            ? "Accepted"
            : verdict === "reject"
              ? "Rejected"
              : "Reviewed"}
        </span>
      {/if}
      {#if reviewedRevision}
        <span>
          Revision {reviewedRevisionNumber} of {revisions.length}
        </span>
        <Id
          id={reviewedRevision.id}
          clipboard={reviewedRevision.id}
          label="revision ID" />
      {/if}
      <span class="timestamp" title={absoluteTimestamp(timestamp)}>
        {formatTimestamp(timestamp)}
      </span>
      {#if isOwnPublishedReview}
        <span style:margin-left="auto"></span>
        <Popover
          popoverPadding="0"
          placement="bottom-end"
          bind:expanded={deleteConfirmExpanded}>
          {#snippet toggle(onclick)}
            <button
              type="button"
              class="action-icon"
              title="Delete review"
              aria-haspopup="dialog"
              aria-expanded={deleteConfirmExpanded}
              disabled={deleting}
              {onclick}>
              <Icon name="trash" />
            </button>
          {/snippet}
          {#snippet popover()}
            <div class="delete-confirm">
              <div class="delete-confirm-message">
                Delete this review? This cannot be undone.
              </div>
              <div class="delete-confirm-actions">
                <Button variant="naked" onclick={() => closeFocused()}>
                  Cancel
                </Button>
                <button
                  type="button"
                  class="delete-confirm-button"
                  onclick={deleteReview}
                  disabled={deleting}>
                  <Icon name="trash" />
                  Delete review
                </button>
              </div>
            </div>
          {/snippet}
        </Popover>
      {/if}
    </div>

    {#if editingSummary}
      <div class="summary-editor">
        <ExtendedTextarea
          {rid}
          body={review.summary ?? ""}
          focus
          submitCaption={savingSummary ? "Saving…" : "Save"}
          disableSubmit={savingSummary}
          submit={async ({ comment }) => {
            await saveSummary(comment);
          }}
          close={() => (editingSummary = false)} />
      </div>
    {:else if summary !== "" || isOwnPublishedReview}
      <div class="summary">
        {#if summary !== ""}
          <Markdown {rid} breaks content={summary} />
        {:else}
          <span class="summary-empty">No summary.</span>
        {/if}
        {#if isOwnPublishedReview}
          <button
            type="button"
            class="action-icon summary-edit"
            title="Edit summary"
            onclick={() => (editingSummary = true)}>
            <Icon name="edit" />
          </button>
        {/if}
      </div>
    {/if}

    <Discussion
      {repoDelegates}
      cobId={patch.id}
      {rid}
      {commentThreads}
      {config}
      createComment={createDiscussionComment}
      editComment={codeActions.editComment}
      reactOnComment={codeActions.reactOnComment} />
  </div>
{/snippet}

<!-- The diff owns the scroll port, and the review's own chrome rides inside it
     as a non-virtualized header, so it scrolls away and leaves the file headers
     pinned — the same arrangement as the Changes tab. -->
<PierreDiff
  patch={loadedDiff?.text ?? ""}
  cacheKeyPrefix={loadedDiff?.key}
  diffStyle={diffOptions.diffStyle}
  wordWrap={diffOptions.wordWrap}
  diffIndicators={diffOptions.indicators}
  lineDiffType={diffOptions.lineDiffType}
  {loadFullFile}
  fileNotes={fileMeta.notes}
  fileStatuses={fileMeta.statuses}
  fileDiffText={reviewedRevision
    ? path =>
        getDiffText(rid, reviewedRevision.base, reviewedRevision.head, 3, path)
    : undefined}
  {collapsedPaths}
  {commentCounts}
  codeComments={reviewCodeComments}
  commentCommit={reviewedRevision?.head}
  header={chrome} />
