<script lang="ts">
  import type { ArtifactNodeStatus } from "@bindings/artifact/ArtifactNodeStatus";

  import { artifactNodeRunning } from "@app/lib/events";
  import { invoke } from "@app/lib/invoke";
  import { hide } from "@app/lib/modal";

  import Button from "@app/components/Button.svelte";
  import Command from "@app/components/Command.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";

  let status = $state<ArtifactNodeStatus | undefined>(undefined);
  let loading = $state(true);
  let error = $state<string | undefined>(undefined);

  // (Re)load whenever the node's running state flips, so opening the modal
  // before the node is up still resolves once it comes online.
  $effect(() => {
    if ($artifactNodeRunning) {
      void refresh();
    } else {
      status = undefined;
      loading = false;
    }
  });

  async function refresh() {
    loading = true;
    error = undefined;
    try {
      status = await invoke<ArtifactNodeStatus>("artifact_node_status");
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
      status = undefined;
    } finally {
      loading = false;
    }
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return "0 B";
    const units = ["B", "KiB", "MiB", "GiB", "TiB"];
    let value = bytes;
    let unit = 0;
    while (value >= 1024 && unit < units.length - 1) {
      value /= 1024;
      unit += 1;
    }
    const formatted =
      value < 10 && unit > 0 ? value.toFixed(1) : value.toFixed(0);
    return `${formatted} ${units[unit]}`;
  }

  function formatUptime(startedAtUnix: number): string {
    const seconds = Math.max(0, Math.floor(Date.now() / 1000) - startedAtUnix);
    const d = Math.floor(seconds / 86400);
    const h = Math.floor((seconds % 86400) / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    if (d > 0) return `${d}d ${h}h`;
    if (h > 0) return `${h}h ${m}m`;
    if (m > 0) return `${m}m`;
    return `${seconds}s`;
  }
</script>

<style>
  .modal {
    width: 40rem;
    max-height: 80vh;
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
    overflow-y: auto;
    padding: 1rem 1.5rem;
  }
  .state {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font: var(--txt-body-m-semibold);
    color: var(--color-text-primary);
    margin-bottom: 1rem;
  }
  .state.offline {
    color: var(--color-text-secondary);
  }
  .dot {
    display: inline-flex;
  }
  .dot.online {
    color: var(--color-feedback-success-text);
  }
  .dot.offline {
    color: var(--color-text-tertiary);
  }
  .grid {
    display: grid;
    grid-template-columns: 10rem 1fr;
    row-gap: 0.5rem;
    column-gap: 1rem;
    align-items: baseline;
  }
  .label {
    font: var(--txt-body-s-regular);
    color: var(--color-text-secondary);
  }
  .value {
    font: var(--txt-body-m-regular);
    color: var(--color-text-primary);
    word-break: break-word;
  }
  .value.mono {
    font: var(--txt-body-m-mono);
  }
  .offline-help {
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
    line-height: 1.625rem;
  }
  .error {
    color: var(--color-feedback-error-text);
    font: var(--txt-body-m-regular);
  }
</style>

<div class="modal">
  <div class="header">
    <span class="title">Artifact node status</span>
    <Button variant="naked" onclick={hide}>
      <span style:color="var(--color-text-tertiary)">
        <Icon name="close" />
      </span>
    </Button>
  </div>

  <div class="body">
    {#if !$artifactNodeRunning}
      <div class="state offline">
        <span class="dot offline"><Icon name="offline" /></span>
        Offline
      </div>
      <div class="offline-help">
        The rad-artifact node is not running. Install <code>rad-artifact</code>
        and start it with:
        <div style:margin-top="0.5rem">
          <Command styleWidth="fit-content" command="rad-artifact node start" />
        </div>
      </div>
    {:else if loading}
      <div class="offline-help">Loading…</div>
    {:else if error}
      <div class="error">{error}</div>
    {:else if status}
      <div class="state">
        <span class="dot online"><Icon name="online" /></span>
        Running · up {formatUptime(status.startedAtUnix)}
      </div>
      <div class="grid">
        <span class="label">Endpoint</span>
        <span class="value mono">
          <Id
            id={status.endpointId}
            clipboard={status.endpointId}
            shorten={false} />
        </span>

        <span class="label">Seeding</span>
        <span class="value">
          {status.seededCount}
          {status.seededCount === 1 ? "artifact" : "artifacts"} ·
          {formatBytes(status.seededBytesLogical)}
        </span>

        <span class="label">Connections</span>
        <span class="value">
          {status.connectionsActive} active · {status.connectionsOpenedTotal} opened
          · {status.connectionsClosedTotal} closed · {status.connectionsDirectTotal}
          direct · {status.holepunchAttempts} holepunch
        </span>

        <span class="label">Paths</span>
        <span class="value">
          {status.pathsDirect} direct · {status.pathsRelayed} relayed
        </span>

        <span class="label">Traffic</span>
        <span class="value">
          ↑ {formatBytes(status.outBytes)} · ↓ {formatBytes(status.inBytes)}
        </span>
      </div>
    {/if}
  </div>
</div>
