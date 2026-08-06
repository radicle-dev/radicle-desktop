<script lang="ts" module>
  import type { CommentOrigin } from "@app/components/Comment.svelte";

  export interface CodeComments {
    changeCommentStatus?: (
      commentId: string,
      resolved: boolean,
    ) => Promise<void>;
    // Whether this thread can be resolved and the current user may do it.
    // Decided by the host: the protocol allows the comment author, the review
    // author or the revision author, and only the host knows all three.
    canResolveComment?: (commentId: string) => boolean;
    config: Config;
    createComment: (
      body: string,
      embeds: Embed[],
      replyTo?: string,
      location?: CodeLocation,
    ) => Promise<void>;
    // Defaults to "Comment".
    newCommentCaption?: string;
    newCommentDescription?: string;
    // When provided, the new-code-comment composer shows a second submit option
    // in the split-button dropdown that posts a `revision.comment` directly
    // (no review wrapping). The primary `createComment` continues to take
    // whatever path the host wires (typically: stash into a draft review).
    addCodeCommentDirect?: (
      body: string,
      embeds: Embed[],
      location: CodeLocation,
    ) => Promise<void>;
    addCodeCommentDirectCaption?: string;
    addCodeCommentDirectDescription?: string;
    editComment: (
      commentId: string,
      body: string,
      embeds: Embed[],
    ) => Promise<void>;
    reactOnComment?: (
      commentId: string,
      authors: Author[],
      reaction: string,
    ) => Promise<void>;
    deleteComment?: (commentId: string) => Promise<void>;
    // Defaults to `true`.
    canReply?: boolean;
    // See `ExtendedTextarea`.
    disableAttachments?: boolean | string;
    // For hosts whose surrounding context already shows the file path.
    hideThreadFileHeader?: boolean;
    // Unpublished draft roots, which render alongside published threads and so
    // have to be told apart.
    draftThreadIds?: string[];
    // Root comment id -> where that thread lives, so comments merged from
    // several sources stay distinguishable in one diff.
    threadOrigins?: Record<string, CommentOrigin>;
    repoDelegates: Author[];
    rid: string;
    threads: Thread<CodeLocation>[];
  }
</script>

<script lang="ts">
  type Side = "left" | "right";
  type SelectionAnchor = { side: Side; lineNumber: number };
  type SelectionRange = { start: SelectionAnchor; end?: SelectionAnchor };

  interface Selection {
    file: string;
    start: SelectionAnchor;
    end: SelectionAnchor;
    lineIdx: number;
    hunkIdx: number;
    codeLocation: CodeLocation;
  }

  import type { Author } from "@bindings/cob/Author";
  import type { CodeLocation } from "@bindings/cob/thread/CodeLocation";
  import type { Embed } from "@bindings/cob/thread/Embed";
  import type { Thread } from "@bindings/cob/thread/Thread";
  import type { Config } from "@bindings/config/Config";
  import type { FileDiff } from "@bindings/diff/FileDiff";
  import type { Modification } from "@bindings/diff/Modification";

  import escape from "lodash/escape";
  import partial from "lodash/partial";

  import * as roles from "@app/lib/roles";

  import CommentToggleInput from "@app/components/CommentToggleInput.svelte";
  import ThreadComponent from "@app/components/Thread.svelte";

  interface Props {
    codeComments?: CodeComments;
    file: FileDiff;
    head: string;
  }

  const { file, head, codeComments }: Props = $props();

  let selection: Selection | undefined = $state(undefined);

  function lineNumber(line: Modification, side: Side): number | undefined {
    if (side === "left") {
      if (line.type === "context") {
        return line.lineNoOld;
      }
      if (line.type === "deletion") {
        return line.lineNo;
      }
    } else {
      if (line.type === "context") {
        return line.lineNoNew;
      }
      if (line.type === "addition") {
        return line.lineNo;
      }
    }
  }

  // A line can carry more than one thread: two reviewers can comment on the
  // same line, and a comment can arrive over the network onto a line we just
  // commented on ourselves. Return all of them, oldest first, so none is
  // shadowed by another.
  function findLineThreads(line: Modification) {
    return (
      (codeComments?.threads ?? [])
        .filter(t => {
          if (line.type === "addition") {
            return t.root.location?.new?.range.end === line.lineNo + 1;
          } else if (line.type === "deletion") {
            return t.root.location?.old?.range.end === line.lineNo + 1;
          } else if (line.type === "context") {
            return (
              t.root.location?.new?.range.end === line.lineNoNew + 1 ||
              t.root.location?.old?.range.end === line.lineNoOld + 1
            );
          }
        })
        .sort((a, b) => a.root.edits[0].timestamp - b.root.edits[0].timestamp)
        // The keyed `each` below throws on a duplicate key, and the thread list
        // is merged from several sources.
        .filter(
          (t, i, all) => all.findIndex(o => o.root.id === t.root.id) === i,
        )
    );
  }

  function determineSelectedAnchor(
    side: Side,
    line: Modification,
  ): SelectionAnchor {
    if (side === "left" && line.type === "addition") {
      return { side: "right", lineNumber: line.lineNo };
    } else if (side === "right" && line.type === "deletion") {
      return { side: "left", lineNumber: line.lineNo };
    } else {
      return side === "left"
        ? { side: "left", lineNumber: lineNumber(line, "left") as number }
        : { side: "right", lineNumber: lineNumber(line, "right") as number };
    }
  }

  function filePath(file: FileDiff, side: Side): string {
    if (file.status === "moved" || file.status === "copied") {
      if (side === "left") {
        return file.oldPath;
      } else {
        return file.newPath;
      }
    } else {
      return file.path;
    }
  }

  function selectLine(
    e: MouseEvent,
    file: FileDiff,
    side: Side,
    line: Modification,
    hunkIdx: number,
    lineIdx: number,
  ) {
    e.preventDefault();
    e.stopPropagation();

    const commentAnchor = determineSelectedAnchor(side, line);

    selection = {
      file: filePath(file, side),
      start: commentAnchor,
      end: commentAnchor,
      hunkIdx: hunkIdx,
      lineIdx: lineIdx,
      codeLocation: {
        commit: head,
        path: filePath(file, side),
        old:
          commentAnchor.side === "left"
            ? {
                type: "lines",
                range: {
                  start: commentAnchor.lineNumber,
                  end: commentAnchor.lineNumber + 1,
                },
              }
            : null,
        new:
          commentAnchor.side === "right"
            ? {
                type: "lines",
                range: {
                  start: commentAnchor.lineNumber,
                  end: commentAnchor.lineNumber + 1,
                },
              }
            : null,
      },
    };
  }

  function isSelected(file: string, hunkIdx: number, lineIdx: number) {
    return (
      selection &&
      selection.file === file &&
      selection.hunkIdx === hunkIdx &&
      selection.lineIdx === lineIdx
    );
  }

  function rangeAnchorsFromCodeLocation(
    location: CodeLocation | null,
  ): SelectionRange | undefined {
    if (location?.old?.type === "lines") {
      return {
        start: { side: "left", lineNumber: location.old.range.start },
      };
    } else if (location?.new?.type === "lines") {
      return {
        start: { side: "right", lineNumber: location.new.range.start },
      };
    }
  }
</script>

<style>
  .container {
    /* Make space for the box-shadow border, otherwise it gets cut off due to
       overflow: hide on the container. */
    padding: 0 0.0625rem 0.5rem;
    font: var(--txt-code-regular);
  }
  .line {
    display: flex;
    position: relative;
    white-space: pre-wrap;
  }
  /* Hover and selection tint the line directly. The previous approach declared
     a `--hover-bg` custom property on every line, which inherits into each of
     its cells, and painted it with a full-bleed inset box-shadow. Both are
     per-line costs on a diff with thousands of lines; these rules only ever
     match the one line being hovered or selected. */
  .line.commentable.context:hover,
  .line.selected.context {
    background-color: color-mix(
      in srgb,
      var(--color-surface-subtle) 85%,
      var(--color-surface-canvas)
    );
  }
  .line.commentable.addition:hover,
  .line.selected.addition {
    background-color: color-mix(
      in srgb,
      var(--color-feedback-success-bg) 92%,
      var(--color-feedback-success-text) 8%
    );
  }
  .line.commentable.deletion:hover,
  .line.selected.deletion {
    background-color: color-mix(
      in srgb,
      var(--color-feedback-error-bg) 92%,
      var(--color-feedback-error-text) 8%
    );
  }
  .hunk-header {
    color: var(--color-text-secondary);
  }
  .hunk-header > .left,
  .hunk-header > .right {
    cursor: default;
  }
  .addition {
    background-color: var(--color-feedback-success-bg);
  }
  .deletion {
    background-color: var(--color-feedback-error-bg);
  }
  .addition > .left,
  .addition > .right,
  .addition > .sign {
    color: var(--color-feedback-success-text);
  }
  .deletion > .left,
  .deletion > .right,
  .deletion > .sign {
    color: var(--color-feedback-error-text);
  }
  .context > .left,
  .context > .right,
  .context > .sign {
    color: var(--color-text-disabled);
  }
  .marker {
    color: var(--color-text-primary) !important;
  }
  .selected {
    z-index: 1;
  }
  .left,
  .right {
    min-width: 3rem;
    text-align: center;
    position: relative;
    cursor: default;
  }
  .comment-add {
    position: absolute;
    right: 0.5rem;
    top: 50%;
    width: 1rem;
    height: 1rem;
    border: 0;
    border-radius: 999px;
    padding: 0;
    background-color: var(--color-surface-brand-secondary);
    color: var(--color-text-on-brand);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    z-index: 2;
    transform: translateY(-50%) scale(0);
    transform-origin: center;
    transition: transform 120ms ease-out;
    pointer-events: none;
  }
  /* The plus is drawn as two bars rather than a "+" glyph, whose optical
     centre depends on the font's metrics. Matches the geometry of the `plus`
     icon: 10x1px within a 16px box. */
  .comment-add::before,
  .comment-add::after {
    content: "";
    position: absolute;
    top: 50%;
    left: 50%;
    background-color: currentColor;
  }
  .comment-add::before {
    width: 0.625rem;
    height: 0.0625rem;
    transform: translate(-50%, -50%);
  }
  .comment-add::after {
    width: 0.0625rem;
    height: 0.625rem;
    transform: translate(-50%, -50%);
  }
  .line.commentable:hover .comment-add {
    transform: translateY(-50%) scale(1);
    pointer-events: auto;
  }
  .comment-add:hover,
  .comment-add:focus-visible {
    background-color: var(--color-brand-hover);
    color: var(--color-text-on-brand);
  }
  .sign {
    min-width: 1.5rem;
  }
  .code {
    -webkit-touch-callout: initial;
    -webkit-user-select: text;
    user-select: text;
    width: 100%;
    word-break: break-word;
    cursor: text;
  }
  .thread {
    background-color: var(--color-surface-base);
    font: var(--txt-body-m-regular);
    padding: 0.5rem;
  }
  .comment-form {
    background-color: var(--color-surface-base);
    display: flex;
    flex-direction: column;
    font: var(--txt-body-m-regular);
    padding: 1rem;
  }
  .comment-header {
    display: flex;
    background-color: var(--color-surface-subtle);
    border-radius: var(--border-radius-sm);
    padding: 0 0.5rem;
    width: fit-content;
  }
</style>

{#snippet lineDiff(line: Modification, lineIdx: number, hunkIdx: number)}
  {@const threads = findLineThreads(line)}
  {@const commentSide = line.type === "deletion" ? "left" : "right"}
  {@const canComment = Boolean(codeComments?.createComment)}
  <div
    class="line"
    class:commentable={canComment}
    class:addition={line.type === "addition"}
    class:deletion={line.type === "deletion"}
    class:context={line.type === "context"}
    class:selected={isSelected(filePath(file, "left"), hunkIdx, lineIdx)}>
    {#if canComment}
      <!-- The glyph is drawn in CSS rather than with <Icon>: this button
           exists on every line of the diff, so an inline SVG here costs two
           extra nodes per line on a patch with thousands of them. -->
      <button
        type="button"
        class="comment-add"
        aria-label="Add a comment on this line"
        title="Add a comment on this line"
        onclick={e => selectLine(e, file, commentSide, line, hunkIdx, lineIdx)}>
      </button>
    {/if}
    <div
      class="left"
      class:marker={selection?.start.side === "left" &&
        selection.start.lineNumber === lineNumber(line, "left")}>
      {lineNumber(line, "left")}
    </div>

    <div
      class="right"
      class:marker={selection?.start.side === "right" &&
        selection.start.lineNumber === lineNumber(line, "right")}>
      {lineNumber(line, "right")}
    </div>

    <div class="sign">
      {#if line.type === "addition"}
        +
      {:else if line.type === "deletion"}
        -
      {/if}
    </div>

    {#if line.highlight && line.highlight.items.length > 0}
      <div class="code">
        {@html line.highlight.items
          .map(
            paint =>
              `<span class="global-syntax ${paint.style}">${escape(paint.item)}</span>`,
          )
          .join("")}
      </div>
    {:else if line.line !== ""}
      <div class="code">{line.line}</div>
    {:else}
      <div class="code"><br /></div>
    {/if}
  </div>

  {#each threads as thread (thread.root.id)}
    {#if codeComments}
      {@const isDraftThread =
        codeComments.draftThreadIds?.includes(thread.root.id) ?? false}
      <!-- Anchors "jump to this comment" from the draft review bar. -->
      <div class="thread" data-thread-id={thread.root.id}>
        {#if !codeComments.hideThreadFileHeader}
          <div class="global-flex" style:padding="0.5rem">
            {@render commentHeader(
              thread.root.location?.path,
              rangeAnchorsFromCodeLocation(thread.root.location),
            )}
          </div>
        {/if}
        <ThreadComponent
          rid={codeComments.rid}
          currentUserNid={codeComments.config.publicKey}
          {thread}
          inline
          draft={isDraftThread}
          origin={codeComments.threadOrigins?.[thread.root.id]}
          reactOnComment={isDraftThread
            ? undefined
            : codeComments.reactOnComment}
          createReply={(codeComments.canReply ?? true)
            ? async (body, embeds) => {
                await codeComments.createComment(body, embeds, thread.root.id);
              }
            : undefined}
          editComment={codeComments.editComment}
          canModifyComment={partial(
            roles.isDelegateOrAuthor,
            codeComments.config.publicKey,
            codeComments.repoDelegates.map(delegate => delegate.did),
          )}
          deleteComment={codeComments.deleteComment}
          changeCommentStatus={isDraftThread
            ? undefined
            : codeComments.changeCommentStatus}
          canResolve={!isDraftThread &&
            Boolean(codeComments.changeCommentStatus) &&
            (codeComments.canResolveComment?.(thread.root.id) ?? false)} />
      </div>
    {/if}
  {/each}

  {#if codeComments && selection?.hunkIdx === hunkIdx && selection?.lineIdx === lineIdx}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="comment-form"
      onpointerdown={e => {
        e.stopPropagation();
      }}>
      <CommentToggleInput
        disallowEmptyBody
        rid={codeComments.rid}
        onclose={() => {
          selection = undefined;
        }}
        focus
        placeholder="Leave a comment"
        submitCaption={codeComments.newCommentCaption}
        submitDescription={codeComments.newCommentDescription}
        disableAttachments={codeComments.disableAttachments}
        submit={async (body, embeds) => {
          if (selection?.codeLocation) {
            try {
              await codeComments.createComment(
                body,
                embeds,
                undefined,
                selection.codeLocation,
              );
            } catch (e) {
              console.error("Comment creation failed", e);
            } finally {
              selection = undefined;
            }
          }
        }}
        secondarySubmit={codeComments.addCodeCommentDirect
          ? {
              caption:
                codeComments.addCodeCommentDirectCaption ?? "Just comment",
              description: codeComments.addCodeCommentDirectDescription,
              submit: async (body, embeds) => {
                if (
                  selection?.codeLocation &&
                  codeComments.addCodeCommentDirect
                ) {
                  try {
                    await codeComments.addCodeCommentDirect(
                      body,
                      embeds,
                      selection.codeLocation,
                    );
                  } catch (e) {
                    console.error("Comment creation failed", e);
                  } finally {
                    selection = undefined;
                  }
                }
              },
            }
          : undefined} />
    </div>
  {/if}
{/snippet}

{#snippet commentHeader(filePath?: string, selectionRange?: SelectionRange)}
  {#if filePath && selectionRange}
    <div class="comment-header">
      {filePath.split("/").length > 1 ? "…/" : ""}{filePath
        .split("/")
        .slice(-1)}:{selectionRange.start.side === "left"
        ? "L"
        : "R"}{selectionRange.start.lineNumber}
      {#if selectionRange.end}
        ->
        {selectionRange.end.side === "left" ? "L" : "R"}{selectionRange.end
          .lineNumber}
      {/if}
    </div>
  {/if}
{/snippet}

<div class="container">
  {#if file.diff.type === "plain"}
    {#each file.diff.hunks as hunk, hunkIdx}
      <div class="line hunk-header">
        <div class="left"></div>
        <div class="right"></div>
        <div class="sign"></div>
        <div class="code">{hunk.header}</div>
      </div>

      <div>
        {#each hunk.lines as modification, lineIdx}
          {@render lineDiff(modification, lineIdx, hunkIdx)}
        {/each}
      </div>
    {/each}
  {/if}
</div>
