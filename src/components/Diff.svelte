<script lang="ts" module>
  export interface CodeComments {
    changeCommentStatus?: (
      commentId: string,
      resolved: boolean,
    ) => Promise<void>;
    config: Config;
    createComment: (
      body: string,
      embeds: Embed[],
      replyTo?: string,
      location?: CodeLocation,
    ) => Promise<void>;
    // Caption for the primary submit button on the new-code-comment composer.
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
    // When `true`, the file:line chip rendered above each inline thread is
    // hidden. Useful when the surrounding context already shows the file path.
    hideThreadFileHeader?: boolean;
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
  import Icon from "@app/components/Icon.svelte";
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

  function findLineThread(line: Modification) {
    return codeComments?.threads.find(t => {
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
    });
  }

  function determineSelectedAnchor(
    side: Side,
    line: Modification,
  ): SelectionAnchor {
    // When a user tries to select a side with no changes, use opposite side.
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

  // eslint-disable-next-line svelte/prefer-writable-derived -- needs a $state proxy so toggleCommentExpand's property mutation triggers reactivity
  let threadExpandedStates: Record<string, boolean> = $state({});

  $effect(() => {
    threadExpandedStates = codeComments
      ? Object.fromEntries(
          codeComments.threads.map(t => [t.root.id, t.root.resolved]),
        )
      : {};
  });

  function toggleCommentExpand(commentId: string) {
    threadExpandedStates[commentId] = !threadExpandedStates[commentId];
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
  .line.commentable:hover {
    box-shadow: inset 0 0 0 9999px var(--hover-bg, transparent);
  }
  .line.commentable.context {
    --hover-bg: color-mix(
      in srgb,
      var(--color-surface-subtle) 85%,
      var(--color-surface-canvas)
    );
  }
  .line.commentable.addition {
    --hover-bg: color-mix(
      in srgb,
      var(--color-feedback-success-bg) 92%,
      var(--color-feedback-success-text) 8%
    );
  }
  .line.commentable.deletion {
    --hover-bg: color-mix(
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
    box-shadow: inset 0 0 0 9999px var(--hover-bg, transparent);
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
  .comment-icon {
    margin-left: auto;
    margin-right: 1rem;
    margin-top: 3px;
    align-self: flex-start;
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
  {@const thread = findLineThread(line)}
  {@const commentSide = line.type === "deletion" ? "left" : "right"}
  <div
    class="line"
    class:commentable={Boolean(codeComments?.createComment && !thread)}
    class:addition={line.type === "addition"}
    class:deletion={line.type === "deletion"}
    class:context={line.type === "context"}
    class:selected={!thread &&
      isSelected(filePath(file, "left"), hunkIdx, lineIdx)}>
    {#if codeComments?.createComment && !thread}
      <button
        type="button"
        class="comment-add"
        title="Add a comment on this line"
        onclick={e => selectLine(e, file, commentSide, line, hunkIdx, lineIdx)}>
        <Icon name="plus" />
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

    <div class="global-flex comment-icon">
      {#if thread}
        <Icon
          name={thread.root.resolved ? "comment-checkmark" : "comment"}
          onclick={() => toggleCommentExpand(thread.root.id)} />
      {/if}
    </div>
  </div>

  {#if codeComments && thread && !threadExpandedStates[thread.root.id]}
    <div class="thread">
      <div class="global-flex" style:padding="0.5rem">
        {#if !codeComments.hideThreadFileHeader}
          {@render commentHeader(
            thread.root.location?.path,
            rangeAnchorsFromCodeLocation(thread.root.location),
          )}
        {/if}
        {#if codeComments.changeCommentStatus && roles.isDelegateOrAuthor( codeComments.config.publicKey, codeComments.repoDelegates.map(delegate => delegate.did), thread.root.author.did, )}
          <div style:margin-left="auto">
            {#if thread.root.resolved}
              <div title="Unresolve comment thread">
                <Icon
                  name="close"
                  onclick={partial(
                    codeComments.changeCommentStatus,
                    thread.root.id,
                    false,
                  )} />
              </div>
            {:else}
              <div title="Resolve comment thread">
                <Icon
                  name="checkmark"
                  onclick={partial(
                    codeComments.changeCommentStatus,
                    thread.root.id,
                    true,
                  )} />
              </div>
            {/if}
          </div>
        {/if}
      </div>
      <ThreadComponent
        rid={codeComments.rid}
        currentUserNid={codeComments.config.publicKey}
        {thread}
        reactOnComment={codeComments.reactOnComment}
        createReply={(codeComments.canReply ?? true)
          ? async (body, embeds) => {
              await codeComments.createComment(body, embeds, thread.root.id);
            }
          : undefined}
        editComment={codeComments.editComment}
        canEditComment={partial(
          roles.isDelegateOrAuthor,
          codeComments.config.publicKey,
          codeComments.repoDelegates.map(delegate => delegate.did),
        )}
        deleteComment={codeComments.deleteComment} />
    </div>
  {/if}

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
