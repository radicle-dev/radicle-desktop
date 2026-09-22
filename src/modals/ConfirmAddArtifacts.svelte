<script lang="ts">
  import type { ArtifactDigest } from "@bindings/cob/release/ArtifactDigest";

  import { disableHide, enableHide, hide } from "@app/lib/modal";
  import { formatBytes } from "@app/lib/utils";

  import Button from "@app/components/Button.svelte";
  import Icon from "@app/components/Icon.svelte";

  export interface StagedArtifact {
    name: string;
    digest: ArtifactDigest;
  }

  interface Props {
    staged: StagedArtifact[];
    /// Whether the artifact node will serve these and announce this node as a
    /// place to fetch them from.
    willSeed: boolean;
    confirm: () => Promise<void>;
  }

  const { staged, willSeed, confirm }: Props = $props();

  const totalBytes = $derived(
    staged.reduce((sum, s) => sum + s.digest.sizeBytes, 0),
  );
  const totalFiles = $derived(
    staged.reduce((sum, s) => sum + s.digest.fileCount, 0),
  );
  const anyDirectory = $derived(staged.some(s => s.digest.directory));

  let working = $state(false);
  let error = $state<string | undefined>(undefined);

  async function run() {
    if (working) return;
    working = true;
    error = undefined;
    // Each artifact is its own signed entry, so the scrim stays inert until
    // the whole batch has settled rather than closing mid-way.
    disableHide();
    try {
      await confirm();
      enableHide();
      hide();
    } catch (e) {
      error = e instanceof Error ? e.message : "Unable to add these artifacts.";
      enableHide();
    } finally {
      working = false;
    }
  }
</script>

<style>
  .modal {
    width: 30rem;
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
    justify-content: space-between;
    padding: 0 1.5rem;
    height: 3.25rem;
    flex-shrink: 0;
    border-bottom: 1px solid var(--color-border-subtle);
  }
  .title {
    font: var(--txt-heading-s);
    color: var(--color-text-primary);
  }
  .body {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    padding: 1.5rem;
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    max-height: 13rem;
    overflow-y: auto;
  }
  .item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
  }
  .item-icon {
    display: inline-flex;
    flex-shrink: 0;
    color: var(--color-text-tertiary);
  }
  .item-name {
    color: var(--color-text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }
  .item-detail {
    margin-left: auto;
    flex-shrink: 0;
    white-space: nowrap;
    color: var(--color-text-tertiary);
    font: var(--txt-body-s-regular);
  }
  .total {
    padding-top: 0.75rem;
    border-top: 1px solid var(--color-border-subtle);
    color: var(--color-text-tertiary);
    font: var(--txt-body-s-regular);
  }
  /* A warning band rather than the quieter note used for reversible actions:
     registering signs a COB entry that syncs to everyone holding the repo, and
     redacting later records a retraction instead of removing it. */
  .warning {
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
    padding: 0.625rem 0.75rem;
    border-radius: var(--border-radius-sm);
    background-color: var(--color-surface-subtle);
  }
  .warning-icon {
    display: inline-flex;
    flex-shrink: 0;
    margin-top: 0.125rem;
    color: var(--color-feedback-warning-text);
  }
  .warning-body {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    min-width: 0;
  }
  .error {
    color: var(--color-feedback-error-text);
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    padding: 0 1.5rem 1.5rem;
  }
  .confirm-label {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
  }
</style>

<div class="modal">
  <div class="header">
    <span class="title">
      Add {staged.length}
      {staged.length === 1 ? "artifact" : "artifacts"}
    </span>
    <Button variant="naked" onclick={hide}>
      <span style:color="var(--color-text-tertiary)">
        <Icon name="close" />
      </span>
    </Button>
  </div>

  <div class="body">
    <div class="list">
      {#each staged as item (item.digest.cid)}
        <div class="item">
          <span class="item-icon">
            <Icon name={item.digest.directory ? "folder" : "attach"} />
          </span>
          <span class="item-name">{item.name}</span>
          <span class="item-detail">
            {#if item.digest.directory}
              {item.digest.fileCount}
              {item.digest.fileCount === 1 ? "file" : "files"} ·
            {/if}
            {formatBytes(item.digest.sizeBytes)}
          </span>
        </div>
      {/each}
    </div>

    {#if staged.length > 1 || anyDirectory}
      <div class="total">
        {totalFiles}
        {totalFiles === 1 ? "file" : "files"} in total, {formatBytes(
          totalBytes,
        )}
      </div>
    {/if}

    <div class="warning">
      <span class="warning-icon"><Icon name="warning" /></span>
      <div class="warning-body">
        <span>
          {#if anyDirectory}
            A folder is added whole, as one artifact named after it. Every file
            inside it counts, including anything you did not mean to include.
          {:else}
            These names and content ids are signed into the release and reach
            everyone who has this repository.
          {/if}
        </span>
        {#if willSeed}
          <span>
            Your artifact node is running, so the contents are served from it
            and this node is announced as a place to fetch them.
          </span>
        {/if}
        <span>
          This cannot be taken back: redacting an artifact later records that it
          should not be used, it does not remove it.
        </span>
      </div>
    </div>

    {#if error}
      <span class="error">{error}</span>
    {/if}
  </div>

  <div class="actions">
    <Button variant="outline" onclick={hide}>Cancel</Button>
    <Button variant="ghost" disabled={working} onclick={() => void run()}>
      <span class="confirm-label">
        <Icon name="plus" />
        {working ? "Adding…" : "Add"}
      </span>
    </Button>
  </div>
</div>
