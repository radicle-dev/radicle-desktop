<script lang="ts">
  import debounce from "lodash/debounce";

  import { writeToClipboard } from "@app/lib/invoke";
  import { hide } from "@app/lib/modal";

  import Button from "@app/components/Button.svelte";
  import Icon from "@app/components/Icon.svelte";
  import ScrollArea from "@app/components/ScrollArea.svelte";

  interface Props {
    raw: string;
    // Which revision the document belongs to, shown so a copied blob can be
    // traced back to a point in the history.
    revision?: string;
  }

  const { raw, revision }: Props = $props();

  let copyIcon: "copy" | "checkmark" = $state("copy");
  const restoreIcon = debounce(() => (copyIcon = "copy"), 1000);

  async function copy() {
    await writeToClipboard(raw);
    copyIcon = "checkmark";
    restoreIcon();
  }
</script>

<style>
  .modal {
    width: min(46rem, 92vw);
    height: min(38rem, 85vh);
    display: flex;
    flex-direction: column;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-lg);
    background-color: var(--color-surface-canvas);
    overflow: hidden;
  }
  .header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0 0.75rem 0 1.5rem;
    height: 3.25rem;
    flex-shrink: 0;
    border-bottom: 1px solid var(--color-border-subtle);
  }
  .title {
    font: var(--txt-heading-s);
    color: var(--color-text-primary);
  }
  .revision {
    font: var(--txt-code-regular);
    color: var(--color-text-tertiary);
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    margin-left: auto;
  }
  pre {
    margin: 0;
    padding: 1.25rem 1.5rem;
    font: var(--txt-code-small);
    color: var(--color-text-primary);
    white-space: pre;
  }
</style>

<div class="modal">
  <div class="header">
    <span class="title">Raw document</span>
    {#if revision}
      <span class="revision">{revision.slice(0, 7)}</span>
    {/if}
    <div class="actions">
      <Button variant="secondary" styleHeight="2rem" onclick={copy}>
        <Icon name={copyIcon} />
        Copy
      </Button>
      <Button variant="naked" onclick={hide}>
        <span style:color="var(--color-text-tertiary)">
          <Icon name="close" />
        </span>
      </Button>
    </div>
  </div>
  <ScrollArea style="flex: 1; min-height: 0;">
    <pre>{raw}</pre>
  </ScrollArea>
</div>
