<script lang="ts" module>
  export interface CodeComments {
    changeCommentStatus: (
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
    editComment: (
      commentId: string,
      body: string,
      embeds: Embed[],
    ) => Promise<void>;
    reactOnComment: (
      publicKey: string,
      commentId: string,
      authors: Author[],
      reaction: string,
    ) => Promise<void>;
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
  import type { Config } from "@bindings/config/Config";
  import type { Embed } from "@bindings/cob/thread/Embed";
  import type { FileDiff } from "@bindings/diff/FileDiff";
  import type { Modification } from "@bindings/diff/Modification";
  import type { Thread } from "@bindings/cob/thread/Thread";

  import * as roles from "@app/lib/roles";

  import escape from "lodash/escape";
  import partial from "lodash/partial";

  import CommentToggleInput from "./CommentToggleInput.svelte";
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

    selection = {
      file: filePath(file, side),
      start: determineSelectedAnchor(side, line),
      end: determineSelectedAnchor(side, line),
      hunkIdx: hunkIdx,
      lineIdx: lineIdx,
      codeLocation: {
        commit: head,
        path: filePath(file, side),
        old:
          side === "left"
            ? {
                type: "lines",
                range: {
                  start: lineNumber(line, "left") as number,
                  end: (lineNumber(line, "left") as number) + 1,
                },
              }
            : null,
        new:
          side === "right"
            ? {
                type: "lines",
                range: {
                  start: lineNumber(line, "right") as number,
                  end: (lineNumber(line, "right") as number) + 1,
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

  let threadExpandedStates: Record<string, boolean> = $state(
    codeComments
      ? Object.fromEntries(
          codeComments.threads.map(t => [t.root.id, t.root.resolved]),
        )
      : {},
  );

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
    padding: 8px 1px;
    font-size: var(--font-size-small);
    font-family: var(--font-family-monospace);
  }
  .line {
    display: flex;
    position: relative;
    white-space: pre-wrap;
  }
  .hunk-header {
    color: var(--color-foreground-dim);
  }
  .hunk-header > .left,
  .hunk-header > .right {
    cursor: default;
  }
  .addition {
    background-color: var(--color-fill-diff-green-light);
  }
  .deletion {
    background-color: var(--color-fill-diff-red-light);
  }
  .addition > .left,
  .addition > .right,
  .addition > .sign {
    color: var(--color-foreground-success);
  }
  .deletion > .left,
  .deletion > .right,
  .deletion > .sign {
    color: var(--color-foreground-red);
  }
  .context > .left,
  .context > .right,
  .context > .sign {
    color: var(--color-foreground-disabled);
  }
  .marker {
    color: var(--color-foreground-contrast) !important;
  }
  .selected {
    box-shadow: 0 0 0 1px var(--color-fill-secondary);
    z-index: 1;
  }
  .left,
  .right {
    min-width: 3rem;
    text-align: center;
    position: relative;
    cursor: cell;
  }
  .selection-disabled {
    cursor: default;
  }
  .left:hover:not(.selection-disabled),
  .right:hover:not(.selection-disabled),
  .left:active:not(.selection-disabled),
  .right:active:not(.selection-disabled) {
    color: var(--color-foreground-contrast);
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
  }
  .comment-icon {
    margin-left: auto;
    margin-right: 1rem;
    margin-top: 3px;
    align-self: flex-start;
  }
  .thread {
    background-color: var(--color-fill-float-hover);
    padding: 0.5rem;
    margin-bottom: 1rem;
  }
  .comment-form {
    background-color: var(--color-fill-float-hover);
    font-family: var(--font-family-sans-serif);
    display: flex;
    flex-direction: column;
    padding: 1rem;
    margin-bottom: 1rem;
  }
  .comment-header {
    display: flex;
    background-color: var(--color-fill-ghost);
    clip-path: var(--1px-corner-fill);
    padding: 0 8px;
    width: fit-content;
  }
</style>

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
        {#each hunk.lines as line, lineIdx}
          {@const thread = findLineThread(line)}
          <div
            class="line"
            class:addition={line.type === "addition"}
            class:deletion={line.type === "deletion"}
            class:context={line.type === "context"}
            class:selected={!thread &&
              isSelected(filePath(file, "left"), hunkIdx, lineIdx)}>
            <div
              class="left"
              class:selection-disabled={!codeComments || thread}
              class:marker={selection?.start.side === "left" &&
                selection.start.lineNumber === lineNumber(line, "left")}
              onpointerdown={e => {
                if (codeComments?.createComment && !thread) {
                  selectLine(e, file, "left", line, hunkIdx, lineIdx);
                }
              }}>
              {lineNumber(line, "left")}
            </div>

            <div
              class="right"
              class:selection-disabled={!codeComments || thread}
              class:marker={selection?.start.side === "right" &&
                selection.start.lineNumber === lineNumber(line, "right")}
              onpointerdown={e => {
                if (codeComments?.createComment && !thread) {
                  selectLine(e, file, "right", line, hunkIdx, lineIdx);
                }
              }}>
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
                {#if thread.root.resolved}
                  <Icon
                    name="comment-checkmark"
                    onclick={() => toggleCommentExpand(thread.root.id)} />
                {:else}
                  <Icon
                    name="comment"
                    onclick={() => toggleCommentExpand(thread.root.id)} />
                {/if}
              {/if}
            </div>
          </div>

          {#if codeComments && thread && !threadExpandedStates[thread.root.id]}
            <div class="thread">
              <div class="global-flex" style:padding="0.5rem">
                {@render commentHeader(
                  thread.root.location?.path,
                  rangeAnchorsFromCodeLocation(thread.root.location),
                )}
                {#if roles.isDelegateOrAuthor( codeComments.config.publicKey, codeComments.repoDelegates.map(delegate => delegate.did), thread.root.author.did, )}
                  <div style:margin-left="auto">
                    {#if thread.root.resolved}
                      <div title="Unresolve comment thread">
                        <Icon
                          name="cross"
                          onclick={() =>
                            codeComments.changeCommentStatus(
                              thread.root.id,
                              false,
                            )} />
                      </div>
                    {:else}
                      <div title="Resolve comment thread">
                        <Icon
                          name="checkmark"
                          onclick={() =>
                            codeComments.changeCommentStatus(
                              thread.root.id,
                              true,
                            )} />
                      </div>
                    {/if}
                  </div>
                {/if}
              </div>
              <ThreadComponent
                inline
                rid={codeComments.rid}
                {thread}
                reactOnComment={codeComments.config &&
                  partial(
                    codeComments.reactOnComment,
                    codeComments.config.publicKey,
                  )}
                createReply={async (body, embeds) => {
                  await codeComments.createComment(
                    body,
                    embeds,
                    thread.root.id,
                  );
                }}
                editComment={codeComments.editComment}
                canEditComment={partial(
                  roles.isDelegateOrAuthor,
                  codeComments.config.publicKey,
                  codeComments.repoDelegates.map(delegate => delegate.did),
                )} />
            </div>
          {/if}

          {#if codeComments && selection && selection.hunkIdx === hunkIdx && selection.lineIdx === lineIdx && selection.codeLocation}
            <div
              class="comment-form"
              onpointerdown={e => {
                e.stopPropagation();
              }}>
              <div style:margin-bottom="1rem">
                {@render commentHeader(selection.file, {
                  start: selection.start,
                })}
              </div>
              <CommentToggleInput
                disallowEmptyBody
                rid={codeComments.rid}
                onclose={() => {
                  selection = undefined;
                }}
                focus
                placeholder="Leave a comment"
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
                }} />
            </div>
          {/if}
        {/each}
      </div>
    {/each}
  {/if}
</div>
