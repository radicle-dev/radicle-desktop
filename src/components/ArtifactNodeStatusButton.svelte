<script lang="ts">
  import type { ArtifactNodeStatus } from "@bindings/artifact/ArtifactNodeStatus";

  import { invoke } from "@app/lib/invoke";
  import { formatBytes } from "@app/lib/utils";

  import Button from "@app/components/Button.svelte";
  import Command from "@app/components/Command.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Popover from "@app/components/Popover.svelte";

  let popoverExpanded: boolean = $state(false);
  let running: boolean | undefined = $state();
  let status: ArtifactNodeStatus | undefined = $state();

  // The button itself reports whether the node answers, so this one poll runs
  // whether or not the popover is open. It is a bare liveness check, unlike the
  // stats below.
  $effect(() => {
    let cancelled = false;

    const refresh = async () => {
      try {
        const running_ = await invoke<boolean>("artifact_node_running");
        if (!cancelled) {
          running = running_;
        }
      } catch {
        if (!cancelled) {
          running = false;
        }
      }
    };

    void refresh();
    const interval = setInterval(() => void refresh(), 5000);

    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  });

  // Only polled while the popover is open: the node is a separate process and
  // its stats are of no use to a collapsed button.
  $effect(() => {
    if (!popoverExpanded) {
      return;
    }
    let cancelled = false;

    const refresh = async () => {
      try {
        const status_ = await invoke<ArtifactNodeStatus>(
          "artifact_node_status",
        );
        if (!cancelled) {
          status = status_;
          running = true;
        }
      } catch {
        if (!cancelled) {
          status = undefined;
          running = false;
        }
      }
    };

    void refresh();
    const interval = setInterval(() => void refresh(), 3000);

    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  });

  const uptime = $derived.by(() => {
    if (!status) {
      return undefined;
    }
    const seconds = Math.max(0, Date.now() / 1000 - status.startedAtUnix);
    if (seconds < 60) {
      return `${Math.floor(seconds)}s`;
    }
    if (seconds < 3600) {
      return `${Math.floor(seconds / 60)}m`;
    }
    if (seconds < 86400) {
      return `${Math.floor(seconds / 3600)}h`;
    }
    return `${Math.floor(seconds / 86400)}d`;
  });
</script>

<style>
  .state {
    margin-left: auto;
    color: var(--color-text-tertiary);
    font: var(--txt-body-s-regular);
  }
  .popover {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    width: 22rem;
    padding: 1rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    background-color: var(--color-surface-canvas);
    font: var(--txt-body-m-regular);
  }
  .stats {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.375rem 1rem;
  }
  .key {
    color: var(--color-text-tertiary);
    white-space: nowrap;
  }
  .value {
    color: var(--color-text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .warning {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: var(--color-feedback-warning-text);
  }
</style>

<Popover
  popoverPadding="0"
  placement="top-start"
  bind:expanded={popoverExpanded}>
  {#snippet toggle(onclick)}
    <Button
      variant="naked"
      {onclick}
      active={popoverExpanded}
      styleWidth="100%"
      styleJustifyContent="flex-start">
      <span style:color="var(--color-text-tertiary)">
        <Icon name="parcel" />
      </span>
      Artifacts
      {#if running !== undefined}
        <span class="state">{running ? "Seeding" : "Offline"}</span>
      {/if}
    </Button>
  {/snippet}
  {#snippet popover()}
    <div class="popover">
      {#if status}
        {#if status.relayUnreachable}
          <div class="warning">
            <Icon name="warning" />
            No relay connected: peers may not reach this node.
          </div>
        {/if}
        <div class="stats">
          <span class="key">Endpoint</span>
          <span class="value" title={status.endpointId}>
            {status.endpointId}
          </span>
          <span class="key">Uptime</span>
          <span class="value">{uptime}</span>
          <span class="key">Seeding</span>
          <span class="value">
            {status.seededCount}
            {status.seededCount === 1 ? "artifact" : "artifacts"}
            ({formatBytes(status.seededBytesLogical)})
          </span>
          <span class="key">Connections</span>
          <span class="value">
            {status.connectionsActive} active, {status.connectionsOpenedTotal} total
          </span>
          <span class="key">Traffic</span>
          <span class="value">
            {formatBytes(status.inBytes)} in, {formatBytes(status.outBytes)} out
          </span>
        </div>
      {:else if running === false}
        <div style:line-height="1.625rem">
          The artifact node is not running, so artifacts cannot be downloaded or
          seeded from this app.
          <div style:margin-top="1rem">
            Start it with:
            <div style:margin-top="0.5rem">
              <Command
                styleWidth="fit-content"
                command="rad-artifact node start" />
            </div>
          </div>
        </div>
      {:else}
        <span class="value">Checking the artifact node…</span>
      {/if}
    </div>
  {/snippet}
</Popover>
