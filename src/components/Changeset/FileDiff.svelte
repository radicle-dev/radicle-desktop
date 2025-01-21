<script lang="ts">
  import type { DiffContent } from "@bindings/diff/DiffContent";
  import type { FileDiff } from "@bindings/diff/FileDiff";
  import type { Modification } from "@bindings/diff/Modification";

  import File from "@app/components/File.svelte";
  import escape from "lodash/escape";

  interface Props {
    filePath: string;
    oldFilePath?: string | undefined;
    fileDiff: DiffContent;
    headerBadgeCaption: FileDiff["status"];
  }

  const {
    filePath,
    oldFilePath = undefined,
    fileDiff,
    headerBadgeCaption,
  }: Props = $props();

  function lineNumberR(line: Modification): string | number {
    switch (line.type) {
      case "addition": {
        return line.lineNo;
      }
      case "context": {
        return line.lineNoNew;
      }
      case "deletion": {
        return " ";
      }
    }
  }

  function lineNumberL(line: Modification): string | number {
    switch (line.type) {
      case "addition": {
        return " ";
      }
      case "context": {
        return line.lineNoOld;
      }
      case "deletion": {
        return line.lineNo;
      }
    }
  }

  function lineSign(line: Modification): string {
    switch (line.type) {
      case "addition": {
        return "+";
      }
      case "context": {
        return " ";
      }
      case "deletion": {
        return "-";
      }
    }
  }
</script>

<style>
  .container {
    font-size: var(--font-size-small);
    overflow-x: auto;
  }
  .actions {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 1rem;
  }
  .browse {
    margin-left: auto;
  }
  .expand-button {
    cursor: pointer;
    user-select: none;
    margin-right: 0.5rem;
  }
  .diff {
    font-family: var(--font-family-monospace);
    table-layout: fixed;
    border-collapse: collapse;
    margin: 0.5rem 0;
  }
  .diff-line {
    vertical-align: top;
  }
  .diff-line.type-addition > * {
    background-color: var(--color-fill-diff-green-light);
  }
  .diff-line.type-deletion > * {
    background-color: var(--color-fill-diff-red-light);
  }

  .diff-line.selected > * {
    background-color: var(--color-fill-float-hover);
  }
  .diff-line.selected.type-addition > * {
    background-color: var(--color-fill-diff-green);
  }
  .diff-line.selected.type-deletion > * {
    background-color: var(--color-fill-diff-red);
  }

  .type-addition > .diff-line-number,
  .type-addition > .diff-line-type {
    color: var(--color-foreground-success);
  }
  .type-deletion > .diff-line-number,
  .type-deletion > .diff-line-type {
    color: var(--color-foreground-red);
  }

  .diff-line.selected .selection-indicator-left {
    background-color: var(--color-fill-secondary);
  }
  .type-addition.diff-line.selected .selection-indicator-left {
    background-color: var(--color-fill-secondary);
  }
  .type-deletion.diff-line.selected .selection-indicator-left {
    background-color: var(--color-fill-secondary);
  }

  .diff-line.selected .selection-indicator-right {
    background-color: var(--color-fill-secondary);
  }
  .type-addition.diff-line.selected .selection-indicator-right {
    background-color: var(--color-fill-secondary);
  }
  .type-deletion.diff-line.selected .selection-indicator-right {
    background-color: var(--color-fill-secondary);
  }

  .selection-start {
    box-shadow: 0 -1px 0 0 var(--color-fill-secondary);
    z-index: 1;
  }
  .selection-end {
    box-shadow: 0 1px 0 0 var(--color-fill-secondary);
    z-index: 1;
  }

  .selection-start.selection-end {
    box-shadow: 0 0 0 1px var(--color-fill-secondary);
    z-index: 1;
  }

  .diff-line-number {
    font-family: var(--font-family-monospace);
    text-align: right;
    user-select: none;
    line-height: 1.5rem;
    min-width: 3rem;
    cursor: pointer;
    color: var(--color-foreground-disabled);
  }
  .diff-line-number.left {
    position: relative;
    padding: 0 0.5rem 0 0.75rem;
  }
  .selection-indicator-left {
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 1px;
  }
  .selection-indicator-right {
    position: absolute;
    right: 0;
    top: 0;
    bottom: 0;
    width: 1px;
  }
  .diff-line-number.right {
    padding: 0 0.75rem 0 0.5rem;
  }
  .diff-line-content {
    color: unset !important;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    width: 100%;
    padding-right: 0.5rem;
  }
  .diff-line-type {
    text-align: center;
    padding-left: 0.75rem;
    padding-right: 0.75rem;
    user-select: none;
  }
  .diff-expand-header {
    padding-left: 0.5rem;
    color: var(--color-foreground-dim);
  }
</style>

<File>
  {#snippet leftHeader()}
    {#if (headerBadgeCaption === "moved" || headerBadgeCaption === "copied") && oldFilePath}
      <span style="display: flex; align-items: center; flex-wrap: wrap;">
        {oldFilePath}
        <span style:padding="0 0.5rem">→</span>
        {filePath}
      </span>
    {:else}
      {filePath}
    {/if}

    {#if headerBadgeCaption === "added"}
      added
    {:else if headerBadgeCaption === "deleted"}
      deleted
    {:else if headerBadgeCaption === "moved"}
      moved
    {:else if headerBadgeCaption === "copied"}
      copied
    {/if}
  {/snippet}

  {#snippet children()}
    <div class="container">
      {#if fileDiff.type === "plain"}
        {#if fileDiff.hunks.length > 0}
          <table class="diff" data-file-diff-select>
            {#each fileDiff.hunks as hunk, hunkIdx}
              <!-- svelte-ignore node_invalid_placement_ssr -->
              <tr class="diff-line hunk-header">
                <td colspan={2} style:position="relative">
                  <div class="selection-indicator-left"></div>
                </td>
                <td
                  colspan={6}
                  class="diff-expand-header"
                  style:position="relative">
                  {hunk.header}
                  <div class="selection-indicator-right"></div>
                </td>
              </tr>
              {#each hunk.lines as line, lineIdx}
                <!-- svelte-ignore node_invalid_placement_ssr -->
                <tr
                  style="position: relative;"
                  class={`diff-line type-${line.type}`}>
                  <td
                    id={[filePath, "H" + hunkIdx, "L" + lineIdx].join("-")}
                    class="diff-line-number left">
                    <div class="selection-indicator-left"></div>
                    {lineNumberL(line)}
                  </td>
                  <td class="diff-line-number right">
                    {lineNumberR(line)}
                  </td>
                  <td class="diff-line-type" data-line-type={line.type}>
                    {lineSign(line)}
                  </td>
                  <td class="diff-line-content">
                    {#if line.highlight}
                      {@html line.highlight.items
                        .map(
                          s =>
                            `<span class="syntax ${s.style}">${escape(s.item)}</span>`,
                        )
                        .join("")}
                    {:else}
                      {line.line}
                    {/if}
                  </td>
                  <td class="selection-indicator-right"></td>
                </tr>
              {/each}
            {/each}
          </table>
        {:else}
          <div style:margin="1rem 0">Empty file</div>
        {/if}
      {:else}
        <div style:margin="1rem 0">Empty file</div>
      {/if}
    </div>
  {/snippet}
</File>
