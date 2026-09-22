<script lang="ts">
  import type { ArtifactProgress } from "@bindings/artifact/ArtifactProgress";
  import type { Artifact } from "@bindings/cob/release/Artifact";
  import type { UnlistenFn } from "@tauri-apps/api/event";

  import { listen } from "@tauri-apps/api/event";

  import { autoSeed, storeAutoSeed } from "@app/lib/autoSeed";
  import { invoke } from "@app/lib/invoke";
  import { formatBytes } from "@app/lib/utils";

  import Button from "@app/components/Button.svelte";
  import Checkbox from "@app/components/Checkbox.svelte";
  import Command from "@app/components/Command.svelte";
  import DelegateBadge from "@app/components/DelegateBadge.svelte";
  import ExternalLink from "@app/components/ExternalLink.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Popover from "@app/components/Popover.svelte";

  interface Props {
    artifact: Artifact;
    delegateIds: Set<string>;
    releaseId: string;
    rid: string;
    /// Whether the node already seeds this artifact, asked of the node rather
    /// than assumed from what this component did.
    seeding: boolean;
    onDownloaded: () => void;
  }

  const {
    artifact,
    delegateIds,
    releaseId,
    rid,
    seeding,
    onDownloaded,
  }: Props = $props();

  let activeTab: "app" | "cli" | "browser" = $state("app");
  // Mirrors the shared preference locally so the checkbox can bind to it,
  // writing back through the store so every artifact row agrees.
  let seedAfterDownload = $state($autoSeed);
  $effect(() => {
    storeAutoSeed(seedAfterDownload);
  });
  let downloading = $state(false);
  let progress: ArtifactProgress | undefined = $state();
  let downloadError: string | undefined = $state();
  let downloaded = $state(false);

  // The node reports byte movement per content id, so a shared event channel
  // is filtered down to this artifact.
  $effect(() => {
    let unlisten: UnlistenFn | undefined;
    let cancelled = false;

    void listen<ArtifactProgress>("artifact_progress", event => {
      if (event.payload.cid === artifact.cid) {
        progress = event.payload;
      }
    }).then(fn => {
      if (cancelled) {
        fn();
      } else {
        unlisten = fn;
      }
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  });

  const percent = $derived.by(() => {
    if (!progress?.total || progress.offset === undefined) {
      return undefined;
    }
    return Math.min(100, Math.round((progress.offset / progress.total) * 100));
  });

  const phaseLabel = $derived.by(() => {
    switch (progress?.phase) {
      case "connecting":
        return "Connecting…";
      case "tryingLocation":
        return "Trying a source…";
      case "locationFailed":
        return "Source failed, trying another…";
      case "downloading":
        return "Downloading…";
      case "exporting":
        return "Writing to disk…";
      default:
        return "Starting…";
    }
  });

  async function download() {
    downloadError = undefined;
    downloaded = false;

    const dest = await invoke<string | null>("pick_artifact_save_path", {
      suggestedName: artifact.name,
    });
    if (!dest) {
      return;
    }

    downloading = true;
    progress = undefined;
    try {
      await invoke("download_artifact", {
        rid,
        releaseId,
        cid: artifact.cid,
        dest,
        seed: seedAfterDownload,
      });
      downloaded = true;
      // Seeding is the node's state, not ours; ask what actually happened.
      onDownloaded();
    } catch {
      downloadError =
        "Download failed. The artifact node may be offline, or no source is reachable.";
    } finally {
      downloading = false;
      progress = undefined;
    }
  }

  // Whether the location can be opened in a browser, as opposed to being
  // served over the radicle-artifact protocol.
  function isWebUrl(url: string): boolean {
    return /^https?:\/\//i.test(url);
  }

  // Delegate locations come first: they are the ones a reader can trust most.
  const webLocations = $derived(
    [...artifact.locations.filter(l => isWebUrl(l.url))].sort(
      (a, b) =>
        Number(delegateIds.has(b.user.did)) -
        Number(delegateIds.has(a.user.did)),
    ),
  );
  // Any location at all can be fetched with the CLI, web locations included.
  const downloadable = $derived(artifact.locations.length > 0);

  // The app tab fetches through the artifact node, so it is only offered while
  // the node answers. The CLI and browser tabs do not need it and stay usable.
  let nodeRunning: boolean | undefined = $state();
  $effect(() => {
    let cancelled = false;

    const refresh = async () => {
      try {
        const running = await invoke<boolean>("artifact_node_running");
        if (!cancelled) {
          nodeRunning = running;
        }
      } catch {
        if (!cancelled) {
          nodeRunning = false;
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
  const command = $derived(
    `rad artifact -r ${rid} download --cid ${artifact.cid}`,
  );
</script>

<style>
  .popover {
    width: 24rem;
    padding: 1rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    background-color: var(--color-surface-canvas);
    font: var(--txt-body-m-regular);
  }
  .tabs {
    display: flex;
    gap: 2px;
    margin-bottom: 1rem;
  }
  label {
    display: block;
    margin-bottom: 0.75rem;
    color: var(--color-text-secondary);
  }
  .warning {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
    color: var(--color-feedback-warning-text);
  }
  .locations {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .location {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
  }
  .url {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--color-text-secondary);
  }
  .progress-track {
    height: 0.25rem;
    border-radius: var(--border-radius-sm);
    background-color: var(--color-surface-subtle);
    overflow: hidden;
    margin-top: 0.5rem;
  }
  .progress-bar {
    height: 100%;
    background-color: var(--color-fill-secondary);
    transition: width 0.1s linear;
  }
  .status {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    margin-top: 0.5rem;
    color: var(--color-text-tertiary);
    font: var(--txt-body-s-regular);
  }
  .error {
    margin-top: 0.75rem;
    color: var(--color-foreground-red);
    font: var(--txt-body-s-regular);
  }
  .success {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-top: 0.75rem;
    color: var(--color-foreground-success);
    font: var(--txt-body-s-regular);
  }
  .seed-option {
    margin-top: 0.75rem;
  }
</style>

<Popover placement="bottom-end" popoverPadding="0">
  {#snippet toggle(onclick)}
    <Button
      {onclick}
      variant="secondary"
      styleHeight="1.75rem"
      disabled={!downloadable}
      title={downloadable
        ? undefined
        : "No download source is currently available for this artifact"}>
      <Icon name="download" />Download
    </Button>
  {/snippet}

  {#snippet popover()}
    <div class="popover">
      <div class="tabs">
        <Button
          styleWidth="100%"
          flatRight
          bordered
          active={activeTab === "app"}
          onclick={() => (activeTab = "app")}>
          <Icon name="download" />App
        </Button>
        <Button
          styleWidth="100%"
          bordered
          flatLeft
          flatRight
          active={activeTab === "cli"}
          onclick={() => (activeTab = "cli")}>
          <Icon name="logo" />CLI
        </Button>
        <Button
          styleWidth="100%"
          flatLeft
          bordered
          disabled={webLocations.length === 0}
          title={webLocations.length === 0
            ? "This artifact has no web locations"
            : undefined}
          active={activeTab === "browser"}
          onclick={() => (activeTab = "browser")}>
          <Icon name="open-external" />Browser
        </Button>
      </div>

      {#if activeTab === "app"}
        <label for="download-artifact">
          {#if nodeRunning === false}
            Your artifact node is not running, so the bytes cannot be fetched or
            verified here. The CLI and browser tabs still work.
          {:else}
            Download through your artifact node, which verifies the bytes
            against the content id.
          {/if}
        </label>
        <Button
          variant="secondary"
          styleWidth="100%"
          disabled={downloading || nodeRunning === false}
          title={nodeRunning === false
            ? "Your artifact node is not running"
            : undefined}
          onclick={download}>
          <Icon name="download" />
          {downloading ? "Downloading…" : "Download"}
        </Button>

        {#if downloading}
          <div class="progress-track">
            <div class="progress-bar" style:width="{percent ?? 0}%"></div>
          </div>
          <div class="status">
            <span>{phaseLabel}</span>
            {#if progress?.offset !== undefined}
              <span>
                {formatBytes(progress.offset)}{progress.total
                  ? ` / ${formatBytes(progress.total)}`
                  : ""}
              </span>
            {/if}
          </div>
        {:else if downloaded}
          <div class="success">
            <Icon name="checkmark" />
            Saved{seeding ? " and seeding" : ""}.
          </div>
        {/if}

        {#if downloadError}
          <div class="error">{downloadError}</div>
        {/if}

        <div class="seed-option">
          <Checkbox bind:checked={seedAfterDownload}>
            Seed after downloading
          </Checkbox>
        </div>
      {:else if activeTab === "cli"}
        <label for="download-command">
          Use the Radicle Artifact CLI to download and verify this artifact.
        </label>
        <Command {command} styleWidth="100%" />
      {:else}
        <div class="warning">
          <Icon name="warning" />
          These downloads are not verified.
        </div>
        <div class="locations">
          {#each webLocations as location (location.url)}
            <div class="location">
              {#if delegateIds.has(location.user.did)}
                <DelegateBadge tooltip="Location added by a delegate" />
              {/if}
              <span class="url" title={location.url}>{location.url}</span>
              <ExternalLink href={location.url} title="Download" />
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/snippet}
</Popover>
