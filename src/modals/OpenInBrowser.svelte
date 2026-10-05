<script lang="ts">
  import { hide } from "@app/lib/modal";

  import Button from "@app/components/Button.svelte";
  import Icon from "@app/components/Icon.svelte";

  interface Props {
    url: string;
    host: string;
  }

  const { url, host }: Props = $props();

  async function open() {
    const { open } = await import("@tauri-apps/plugin-shell");
    await open(url);
    hide();
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
  .url {
    font: var(--txt-code-regular);
    color: var(--color-text-tertiary);
    word-break: break-all;
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
    <span class="title">Open in browser</span>
    <Button variant="naked" onclick={hide}>
      <span style:color="var(--color-text-tertiary)">
        <Icon name="close" />
      </span>
    </Button>
  </div>

  <div class="body">
    <span>
      A link was opened for something that isn't on this node. It can be viewed
      on {host} instead.
    </span>
    <span class="url">{url}</span>
  </div>

  <div class="actions">
    <Button variant="outline" onclick={hide}>Cancel</Button>
    <Button variant="ghost" onclick={() => void open()}>
      <span class="confirm-label">
        <Icon name="open-external" />
        Open in {host}
      </span>
    </Button>
  </div>
</div>
