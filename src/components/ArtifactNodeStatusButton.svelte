<script lang="ts">
  import type { ArtifactBinaries } from "@bindings/artifact/ArtifactBinaries";
  import type { ArtifactNodeStatus } from "@bindings/artifact/ArtifactNodeStatus";

  import { invoke } from "@app/lib/invoke";
  import { formatBytes } from "@app/lib/utils";

  import Button from "@app/components/Button.svelte";
  import Command from "@app/components/Command.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import Popover from "@app/components/Popover.svelte";

  // Installs both `rad-artifact` and `rad-artifact-node`.
  const INSTALL_COMMAND =
    "curl -sSf https://files.radicle.dev/releases/radicle-artifact/install | sh";

  let popoverExpanded: boolean = $state(false);
  let running: boolean | undefined = $state();
  let status: ArtifactNodeStatus | undefined = $state();
  let binaries: ArtifactBinaries | undefined = $state();

  // A node that is down is a different problem from one that was never
  // installed, and telling someone to run a binary they do not have is a dead
  // end. Both binaries are needed: `rad-artifact node start` only spawns the
  // daemon, which is the separate `rad-artifact-node`. Undefined while the
  // check is outstanding, so the guidance below does not flash "install" at
  // someone who has it.
  const installed = $derived(
    binaries === undefined ? undefined : binaries.cli && binaries.node,
  );

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
        const binaries_ = await invoke<ArtifactBinaries>("artifact_binaries");
        if (!cancelled) {
          binaries = binaries_;
        }
      } catch {
        if (!cancelled) {
          binaries = undefined;
        }
      }

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
  /* The endpoint is an address to hand to someone else, so it is shown whole
     and wraps rather than being truncated to a prefix that identifies
     nothing. */
  .endpoint {
    color: var(--color-text-secondary);
    word-break: break-all;
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
          <span class="endpoint">
            <Id
              id={status.endpointId}
              clipboard={status.endpointId}
              label="endpoint"
              shorten={false} />
          </span>
          <span class="key">Uptime</span>
          <span class="value">{uptime}</span>
          <span class="key">Seeding</span>
          <span class="value">
            {status.seededCount}
            {status.seededCount === 1 ? "artifact" : "artifacts"}
            ({formatBytes(status.seededBytesLogical)})
          </span>
          <!-- The node keeps its own copy of everything it serves, so a
               download leaves bytes here as well as at the path the user
               picked. Worth saying where, since deleting the saved file does
               not reclaim this one. -->
          <span class="key">Store</span>
          <span class="endpoint">
            <Id
              id={status.storePath}
              clipboard={status.storePath}
              label="store path"
              shorten={false} />
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
      {:else if running === false && installed !== undefined}
        <div style:line-height="1.625rem">
          {#if installed}
            The artifact node is not running, so artifacts cannot be downloaded
            or seeded from this app.
          {:else if binaries?.cli}
            The artifact seeding daemon is not installed, so artifacts cannot be
            downloaded or seeded from this app.
          {:else}
            The artifact tools are not installed, so artifacts cannot be
            downloaded or seeded from this app.
          {/if}
          <div style:margin-top="1rem">
            {installed ? "Start it with:" : "Install them with:"}
            <div style:margin-top="0.5rem">
              {#if installed}
                <Command
                  styleWidth="fit-content"
                  command="rad-artifact node start" />
              {:else}
                <Command styleWidth="100%" command={INSTALL_COMMAND} />
              {/if}
            </div>
          </div>
        </div>
      {:else}
        <span class="value">Checking the artifact node…</span>
      {/if}
    </div>
  {/snippet}
</Popover>
