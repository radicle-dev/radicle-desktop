<script lang="ts">
  import type { Modification } from "@bindings/diff/Modification";

  import escape from "lodash/escape";
  import partial from "lodash/partial";

  import { binaryImageUrl } from "@app/lib/binaryImage";
  import type { CommentContext, SelectionRange } from "@app/lib/diffComments";
  import {
    findLineThread,
    lineNumber,
    rangeAnchorsFromCodeLocation,
  } from "@app/lib/diffComments";
  import type { DiffRow } from "@app/lib/diffRows";
  import { fileDiffName, fileDiffPath } from "@app/lib/diffText";
  import { getDiffText } from "@app/lib/invoke";
  import * as roles from "@app/lib/roles";

  import CommentToggleInput from "@app/components/CommentToggleInput.svelte";
  import DiffActions from "@app/components/DiffActions.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Path from "@app/components/Path.svelte";
  import ThreadComponent from "@app/components/Thread.svelte";

  interface Props {
    row: DiffRow;
    // Whether the row's file is expanded (only meaningful for file-header rows).
    expanded?: boolean;
    onToggleFile?: (fileIndex: number) => void;
    // Present only on the patch-review path; enables inline comments/selection.
    comments?: CommentContext;
    // Identifies the commit range this row's diff belongs to, so file-level
    // actions can ask the backend for patch text.
    diffContext: { rid: string; base?: string; head: string; unified: number };
  }

  const {
    row,
    expanded = true,
    onToggleFile,
    comments,
    diffContext,
  }: Props = $props();

  type LineRow = Extract<DiffRow, { type: "line" }>;
</script>

<style>
  .file-gap {
    height: 1rem;
  }
  .file-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    height: 2.5rem;
    /* Horizontal inset of the diff cards. The container sets `--diff-inset`
       (e.g. 0 when it already provides its own side padding); falls back to 1rem
       when the diff is the full-width scroll content (commit view). */
    margin: 0 var(--diff-inset, 1rem);
    padding: 0 0.5rem;
    background-color: var(--color-surface-canvas);
    /* Bottom border present in both states: it's the separator above the lines
       when expanded, and the card bottom when standalone. Keeping it always
       avoids a 1px reflow when toggling collapse. */
    border: 1px solid var(--color-border-subtle);
    border-top-left-radius: var(--border-radius-md);
    border-top-right-radius: var(--border-radius-md);
    font: var(--txt-body-m-regular);
  }
  .file-header.standalone {
    border-bottom-left-radius: var(--border-radius-md);
    border-bottom-right-radius: var(--border-radius-md);
  }
  .collapse {
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    color: var(--color-text-secondary);
    display: flex;
    align-items: center;
  }
  .collapse:hover {
    color: var(--color-text-primary);
  }
  .arrow {
    padding: 0 0.25rem;
    color: var(--color-text-secondary);
  }
  .status {
    padding: 0 0.375rem;
    border-radius: var(--border-radius-sm);
  }
  .status.added {
    color: var(--color-feedback-success-text);
    background-color: var(--color-feedback-success-bg);
  }
  .status.deleted {
    color: var(--color-feedback-error-text);
    background-color: var(--color-feedback-error-bg);
  }
  .status.moved,
  .status.copied {
    color: var(--color-text-secondary);
    background: var(--color-surface-subtle);
  }
  .stats {
    margin-left: auto;
    font: var(--txt-code-regular);
    white-space: nowrap;
  }
  .add {
    color: var(--color-feedback-success-text);
  }
  .del {
    color: var(--color-feedback-error-text);
  }

  .line {
    display: flex;
    position: relative;
    white-space: pre-wrap;
    margin: 0 var(--diff-inset, 1rem);
    font: var(--txt-code-regular);
    border-left: 1px solid var(--color-border-subtle);
    border-right: 1px solid var(--color-border-subtle);
    background-color: var(--color-surface-canvas);
  }
  .line.last {
    border-bottom: 1px solid var(--color-border-subtle);
    border-bottom-left-radius: var(--border-radius-md);
    border-bottom-right-radius: var(--border-radius-md);
  }
  .line.selected {
    box-shadow: 0 0 0 1px var(--color-border-brand);
    z-index: 1;
  }
  .hunk-header {
    color: var(--color-text-secondary);
  }
  .file-note {
    justify-content: center;
    align-items: center;
    gap: 0.5rem;
    padding: 1rem 0;
    font: var(--txt-body-m-regular);
  }
  .binary-image {
    max-width: 100%;
    max-height: 32rem;
    object-fit: contain;
    display: block;
  }
  .addition {
    background-color: var(--color-feedback-success-bg);
  }
  .deletion {
    background-color: var(--color-feedback-error-bg);
  }
  .addition > .num,
  .addition > .sign {
    color: var(--color-feedback-success-text);
  }
  .deletion > .num,
  .deletion > .sign {
    color: var(--color-feedback-error-text);
  }
  .context > .num,
  .context > .sign {
    color: var(--color-text-disabled);
  }
  .num {
    min-width: 3rem;
    text-align: center;
    flex-shrink: 0;
  }
  .num.gutter:not(.selection-disabled) {
    cursor: cell;
  }
  .num.gutter:not(.selection-disabled):hover {
    color: var(--color-text-primary);
  }
  .marker {
    color: var(--color-text-primary) !important;
  }
  .sign {
    min-width: 1.5rem;
    flex-shrink: 0;
  }
  .code {
    -webkit-user-select: text;
    user-select: text;
    width: 100%;
    word-break: break-word;
    padding-right: 1rem;
  }
  .comment-icon {
    margin-left: auto;
    margin-right: 1rem;
    margin-top: 3px;
    align-self: flex-start;
  }
  .thread,
  .comment-form {
    margin: 0 var(--diff-inset, 1rem);
    border-left: 1px solid var(--color-border-subtle);
    border-right: 1px solid var(--color-border-subtle);
    background-color: var(--color-surface-base);
    font: var(--txt-body-m-regular);
  }
  .thread {
    padding: 0.5rem;
  }
  .comment-form {
    display: flex;
    flex-direction: column;
    padding: 1rem;
  }
  .thread.last,
  .comment-form.last {
    border-bottom: 1px solid var(--color-border-subtle);
    border-bottom-left-radius: var(--border-radius-md);
    border-bottom-right-radius: var(--border-radius-md);
  }
  .comment-header {
    display: flex;
    background-color: var(--color-surface-subtle);
    border-radius: var(--border-radius-sm);
    padding: 0 0.5rem;
    width: fit-content;
  }
</style>

{#snippet code(line: Modification)}
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

{#snippet readonlyLine(r: LineRow)}
  {@const line = r.modification}
  <div
    class="line"
    class:addition={line.type === "addition"}
    class:deletion={line.type === "deletion"}
    class:context={line.type === "context"}
    class:last={r.last}>
    <div class="num">{lineNumber(line, "left") ?? ""}</div>
    <div class="num">{lineNumber(line, "right") ?? ""}</div>
    <div class="sign">
      {#if line.type === "addition"}+{:else if line.type === "deletion"}-{/if}
    </div>
    {@render code(line)}
  </div>
{/snippet}

{#snippet interactiveLine(r: LineRow, ctx: CommentContext)}
  {@const line = r.modification}
  {@const cc = ctx.codeComments}
  {@const thread = findLineThread(ctx.threadByLine, r.file, line)}
  {@const showThread =
    thread !== undefined && !ctx.expandedStates[thread.root.id]}
  {@const sel = ctx.selection}
  {@const inFile = sel !== undefined && sel.fileIdx === r.fileIndex}
  {@const isSel =
    inFile && sel.hunkIdx === r.hunkIndex && sel.lineIdx === r.lineIndex}
  {@const closeOnLine = r.last && !showThread && !isSel}
  {@const closeOnThread = r.last && showThread && !isSel}
  {@const closeOnForm = r.last && isSel}
  {@const selectable = cc.createComment !== undefined && thread === undefined}
  <div
    class="line"
    class:addition={line.type === "addition"}
    class:deletion={line.type === "deletion"}
    class:context={line.type === "context"}
    class:selected={thread === undefined && isSel}
    class:last={closeOnLine}>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="num gutter"
      class:selection-disabled={!selectable}
      class:marker={inFile &&
        sel.start.side === "left" &&
        sel.start.lineNumber === lineNumber(line, "left")}
      onpointerdown={e => {
        if (selectable) {
          e.preventDefault();
          e.stopPropagation();
          ctx.onSelectLine(
            r.fileIndex,
            "left",
            line,
            r.hunkIndex,
            r.lineIndex,
            r.file,
          );
        }
      }}>
      {lineNumber(line, "left") ?? ""}
    </div>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="num gutter"
      class:selection-disabled={!selectable}
      class:marker={inFile &&
        sel.start.side === "right" &&
        sel.start.lineNumber === lineNumber(line, "right")}
      onpointerdown={e => {
        if (selectable) {
          e.preventDefault();
          e.stopPropagation();
          ctx.onSelectLine(
            r.fileIndex,
            "right",
            line,
            r.hunkIndex,
            r.lineIndex,
            r.file,
          );
        }
      }}>
      {lineNumber(line, "right") ?? ""}
    </div>
    <div class="sign">
      {#if line.type === "addition"}+{:else if line.type === "deletion"}-{/if}
    </div>
    {@render code(line)}
    {#if thread}
      <div class="global-flex comment-icon">
        <Icon
          name={thread.root.resolved ? "comment-checkmark" : "comment-cross"}
          onclick={() => ctx.onToggleThread(thread.root.id)} />
      </div>
    {/if}
  </div>

  {#if showThread && thread}
    <div class="thread" class:last={closeOnThread}>
      <div class="global-flex" style:padding="0.5rem">
        {@render commentHeader(
          thread.root.location?.path,
          rangeAnchorsFromCodeLocation(thread.root.location),
        )}
        {#if cc.changeCommentStatus && roles.isDelegateOrAuthor( cc.config.publicKey, cc.repoDelegates.map(delegate => delegate.did), thread.root.author.did, )}
          <div style:margin-left="auto">
            {#if thread.root.resolved}
              <div title="Unresolve comment thread">
                <Icon
                  name="close"
                  onclick={partial(
                    cc.changeCommentStatus,
                    thread.root.id,
                    false,
                  )} />
              </div>
            {:else}
              <div title="Resolve comment thread">
                <Icon
                  name="checkmark"
                  onclick={partial(
                    cc.changeCommentStatus,
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
        rid={cc.rid}
        currentUserNid={cc.config.publicKey}
        {thread}
        reactOnComment={cc.reactOnComment}
        createReply={(cc.canReply ?? true)
          ? async (body, embeds) => {
              await cc.createComment(body, embeds, thread.root.id);
            }
          : undefined}
        editComment={cc.editComment}
        canEditComment={partial(
          roles.isDelegateOrAuthor,
          cc.config.publicKey,
          cc.repoDelegates.map(delegate => delegate.did),
        )}
        deleteComment={cc.deleteComment} />
    </div>
  {/if}

  {#if isSel && sel}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="comment-form"
      class:last={closeOnForm}
      onpointerdown={e => {
        e.stopPropagation();
      }}>
      <div style:margin-bottom="1rem">
        {@render commentHeader(sel.file, { start: sel.start })}
      </div>
      <CommentToggleInput
        disallowEmptyBody
        rid={cc.rid}
        onclose={() => ctx.onClearSelection()}
        focus
        placeholder="Leave a comment"
        disableAttachments={cc.disableAttachments}
        submit={async (body, embeds) => {
          try {
            await cc.createComment(body, embeds, undefined, sel.codeLocation);
          } catch (e) {
            console.error("Comment creation failed", e);
          } finally {
            ctx.onClearSelection();
          }
        }} />
    </div>
  {/if}
{/snippet}

{#if row.type === "file-gap"}
  <div class="file-gap"></div>
{:else if row.type === "file-header"}
  <div class="file-header" class:standalone={row.standalone}>
    <button
      class="collapse"
      onclick={() => onToggleFile?.(row.fileIndex)}
      title={expanded ? "Collapse file" : "Expand file"}>
      <Icon name={expanded ? "chevron-down" : "chevron-right"} />
    </button>
    {#if row.file.status === "moved" || row.file.status === "copied"}
      <Path fullPath={row.file.oldPath} />
      <span class="arrow">→</span>
      <Path fullPath={row.file.newPath} />
    {:else}
      <Path fullPath={row.file.path} />
    {/if}
    {#if row.file.status === "added"}
      <span class="status added">Added</span>
    {:else if row.file.status === "deleted"}
      <span class="status deleted">Deleted</span>
    {:else if row.file.status === "moved"}
      <span class="status moved">Moved</span>
    {:else if row.file.status === "copied"}
      <span class="status copied">Copied</span>
    {/if}

    <span class="stats">
      {#if row.file.diff.type === "plain" && row.file.diff.hunks.length > 0}
        <span class="add">+{row.file.diff.stats.additions}</span>
        <span class="del">-{row.file.diff.stats.deletions}</span>
      {/if}
    </span>
    {#if comments && (row.file.status === "added" || row.file.status === "deleted" || row.file.status === "modified")}
      {@const counts = comments.threadCountsByFile.get(row.file.path)}
      {#if counts !== undefined && counts.unresolved > 0}
        <div class="global-flex">
          <Icon name="comment-cross" />
          {counts.unresolved}
        </div>
      {/if}
      {#if counts !== undefined && counts.resolved > 0}
        <div class="global-flex">
          <Icon name="comment-checkmark" />
          {counts.resolved}
        </div>
      {/if}
    {/if}
    <DiffActions
      text={() =>
        getDiffText(
          diffContext.rid,
          diffContext.base,
          diffContext.head,
          diffContext.unified,
          fileDiffPath(row.file),
        )}
      fileName={fileDiffName(row.file)}
      title="File diff actions" />
  </div>
{:else if row.type === "hunk-header"}
  <div class="line hunk-header">
    <div class="num"></div>
    <div class="num"></div>
    <div class="sign"></div>
    <div class="code">{row.header}</div>
  </div>
{:else if row.type === "file-note"}
  <div class="line last file-note">
    {#if row.note === "binary"}
      {#await binaryImageUrl(diffContext.rid, row.file)}
        <Icon name="binary" />
        Binary file
      {:then url}
        {#if url}
          <img class="binary-image" src={url} alt={fileDiffPath(row.file)} />
        {:else}
          <Icon name="binary" />
          Binary file
        {/if}
      {:catch}
        <Icon name="binary" />
        Binary file
      {/await}
    {:else}
      Empty file
    {/if}
  </div>
{:else if comments}
  {@render interactiveLine(row, comments)}
{:else}
  {@render readonlyLine(row)}
{/if}
