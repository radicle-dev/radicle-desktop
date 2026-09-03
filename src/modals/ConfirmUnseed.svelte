<script lang="ts">
  import { disableHide, enableHide, hide } from "@app/lib/modal";
  import { repoListScope } from "@app/lib/repoListScope";

  import Button from "@app/components/Button.svelte";
  import Checkbox from "@app/components/Checkbox.svelte";
  import Icon from "@app/components/Icon.svelte";

  interface Props {
    name: string;
    rid: string;
    confirm: (clean: boolean) => Promise<void>;
  }

  const { name, rid, confirm }: Props = $props();

  let clean = $state(false);
  let working = $state(false);
  let error = $state<string | undefined>(undefined);

  async function run() {
    if (working) return;
    working = true;
    error = undefined;
    // Two commands run back to back, so the scrim and the close button stay
    // inert until both have settled.
    disableHide();
    try {
      await confirm(clean);
      enableHide();
      hide();
    } catch (e) {
      error = e instanceof Error ? e.message : "Unable to stop seeding.";
      enableHide();
    } finally {
      working = false;
    }
  }
</script>

<style>
  .modal {
    width: 26rem;
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
  .repo {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font: var(--txt-body-m-medium);
    color: var(--color-text-primary);
    min-width: 0;
  }
  .repo-icon {
    display: inline-flex;
    color: var(--color-text-tertiary);
    flex-shrink: 0;
  }
  .rid {
    font: var(--txt-code-regular);
    color: var(--color-text-tertiary);
    word-break: break-all;
  }
  /* Deliberately not a warning band: `rad unseed` drops the seeding policy and
     nothing else, so this is reversible and loses no data. */
  .note {
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
    padding: 0.625rem 0.75rem;
    border-radius: var(--border-radius-sm);
    background-color: var(--color-surface-subtle);
    color: var(--color-text-secondary);
  }
  .note-icon {
    display: inline-flex;
    flex-shrink: 0;
    margin-top: 0.125rem;
    color: var(--color-text-tertiary);
  }
  .note-body {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
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
    <span class="title">Stop seeding</span>
    <Button variant="naked" onclick={hide}>
      <span style:color="var(--color-text-tertiary)">
        <Icon name="close" />
      </span>
    </Button>
  </div>

  <div class="body">
    <div class="repo txt-overflow">
      <span class="repo-icon"><Icon name="repository" /></span>
      {name}
    </div>
    <span class="rid">{rid}</span>

    <div class="note txt-body-m-regular">
      <span class="note-icon"><Icon name="seed" /></span>
      <div class="note-body">
        <span>
          Your node stops replicating and announcing this repository{repoListScope.value ===
          "seeded"
            ? ", and it leaves the sidebar."
            : "; the sidebar dims it."}
          {#if !clean}
            The files stay on disk.
          {/if}
        </span>
        <!-- Its own paragraph rather than a swapped-in sentence, so ticking the
             box reads as text appearing instead of a block reflowing. -->
        {#if clean}
          <span>
            The files go too, except the refs your node signed and the
            delegates'. Other nodes have already fetched yours, so replacing
            them later would fork your own history, and the repository can't be
            verified without the delegates'. If you have never published here
            there is nothing signed, and it is deleted outright.
          </span>
        {/if}
        <Checkbox bind:checked={clean} disabled={working}>
          Delete the files from local storage
        </Checkbox>
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
        <Icon name={clean ? "trash" : "seed"} />
        {working ? "Stopping…" : "Stop seeding"}
      </span>
    </Button>
  </div>
</div>
