<script lang="ts">
  import type { Modification } from "@bindings/diff/Modification";

  import escape from "lodash/escape";

  import type { DiffRow } from "@app/lib/diffRows";
  import { fileDiffToText } from "@app/lib/diffText";

  import CopyButton from "@app/components/CopyButton.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Path from "@app/components/Path.svelte";

  interface Props {
    row: DiffRow;
    // Whether the row's file is expanded (only meaningful for file-header rows).
    expanded?: boolean;
    onToggleFile?: (fileIndex: number) => void;
  }

  const { row, expanded = true, onToggleFile }: Props = $props();

  function lineNumber(line: Modification, side: "left" | "right") {
    if (side === "left") {
      if (line.type === "context") return line.lineNoOld;
      if (line.type === "deletion") return line.lineNo;
    } else {
      if (line.type === "context") return line.lineNoNew;
      if (line.type === "addition") return line.lineNo;
    }
  }
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
    margin: 0 1rem;
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
    margin: 0 1rem;
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
</style>

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
    <CopyButton text={() => fileDiffToText(row.file)} title="Copy file diff" />
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
      <Icon name="binary" />
      Binary file
    {:else}
      Empty file
    {/if}
  </div>
{:else}
  {@const line = row.modification}
  <div
    class="line"
    class:addition={line.type === "addition"}
    class:deletion={line.type === "deletion"}
    class:context={line.type === "context"}
    class:last={row.last}>
    <div class="num">{lineNumber(line, "left") ?? ""}</div>
    <div class="num">{lineNumber(line, "right") ?? ""}</div>
    <div class="sign">
      {#if line.type === "addition"}+{:else if line.type === "deletion"}-{/if}
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
{/if}
