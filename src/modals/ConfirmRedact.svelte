<script lang="ts">
  import { disableHide, enableHide, hide } from "@app/lib/modal";

  import Button from "@app/components/Button.svelte";
  import Icon from "@app/components/Icon.svelte";
  import TextInput from "@app/components/TextInput.svelte";

  interface Props {
    name: string;
    /// Whether this node is currently serving the bytes, which redacting also
    /// stops.
    seeding: boolean;
    confirm: (reason: string) => Promise<void>;
  }

  const { name, seeding, confirm }: Props = $props();

  let reason = $state("");
  let working = $state(false);
  let error = $state<string | undefined>(undefined);

  // A redaction is read by other people deciding whether to trust what they
  // already downloaded, so it is worth nothing without a reason.
  const ready = $derived(reason.trim().length > 0);

  async function run() {
    if (working || !ready) return;
    working = true;
    error = undefined;
    disableHide();
    try {
      await confirm(reason.trim());
      enableHide();
      hide();
    } catch (e) {
      error =
        e instanceof Error ? e.message : "Unable to redact this artifact.";
      enableHide();
    } finally {
      working = false;
    }
  }
</script>

<style>
  .modal {
    width: 28rem;
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
  .artifact {
    color: var(--color-text-primary);
    word-break: break-all;
  }
  .label {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }
  /* A warning band: the record is permanent and public, and it costs this node
     the ability to attest to the artifact afterwards. */
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
    <span class="title">Redact artifact</span>
    <Button variant="naked" onclick={hide}>
      <span style:color="var(--color-text-tertiary)">
        <Icon name="close" />
      </span>
    </Button>
  </div>

  <div class="body">
    <span class="artifact">{name}</span>

    <span>
      This marks the artifact as one that should no longer be used, and hides it
      from the release by default.
    </span>

    <label class="label">
      Reason
      <TextInput
        autofocus
        name="redact-reason"
        placeholder="Why should this no longer be used?"
        disabled={working}
        onSubmit={() => void run()}
        bind:value={reason} />
    </label>

    <div class="warning">
      <span class="warning-icon"><Icon name="warning" /></span>
      <div class="warning-body">
        <span>
          It does not delete anything. The content id stays in the release's
          history, and anyone who already has the bytes keeps them.
        </span>
        {#if seeding}
          <span>
            Your node stops serving it and withdraws the location it announced.
          </span>
        {/if}
        <span>
          This cannot be undone, and you will not be able to attest to this
          artifact afterwards.
        </span>
      </div>
    </div>

    {#if error}
      <span class="error">{error}</span>
    {/if}
  </div>

  <div class="actions">
    <Button variant="outline" onclick={hide}>Cancel</Button>
    <Button
      variant="ghost"
      disabled={working || !ready}
      onclick={() => void run()}>
      <span class="confirm-label">
        <Icon name="warning" />
        {working ? "Redacting…" : "Redact"}
      </span>
    </Button>
  </div>
</div>
