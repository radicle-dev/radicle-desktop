<script lang="ts">
  import type { ReleaseFilter } from "@app/views/repo/router";
  import type { Artifact } from "@bindings/cob/release/Artifact";
  import type { Release } from "@bindings/cob/release/Release";
  import type { Config } from "@bindings/config/Config";
  import type { Commit } from "@bindings/repo/Commit";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import { listen } from "@tauri-apps/api/event";
  import { get } from "svelte/store";
  import { onDestroy, onMount } from "svelte";

  import { autoSeedArtifacts } from "@app/lib/autoSeed";
  import { artifactNodeRunning } from "@app/lib/events";
  import { invoke, InvokeError } from "@app/lib/invoke";
  import { isDelegateOrAuthor } from "@app/lib/roles";
  import * as router from "@app/lib/router";
  import {
    absoluteTimestamp,
    authorForNodeId,
    formatTimestamp,
    publicKeyFromDid,
  } from "@app/lib/utils";

  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import NodeId from "@app/components/NodeId.svelte";
  import ScrollArea from "@app/components/ScrollArea.svelte";
  import Topbar from "@app/components/Topbar.svelte";

  import Layout from "./Layout.svelte";

  interface Props {
    repo: RepoInfo;
    release: Release;
    config: Config;
    filter: ReleaseFilter;
  }

  const { repo, release: releaseProp, config, filter }: Props = $props();

  let release = $state(releaseProp);
  $effect(() => {
    release = releaseProp;
  });

  // Resolve the released commit for the description panel. Lookup is best
  // effort; if the commit was garbage collected we just hide the panel.
  let commit = $state<Commit | undefined>(undefined);
  $effect(() => {
    const oid = release.oid;
    void invoke<Commit>("repo_commit", { rid: repo.rid, sha: oid })
      .then(c => {
        commit = c;
      })
      .catch(() => {
        commit = undefined;
      });
  });

  async function refresh() {
    const next = await invoke<Release | null>("release_by_id", {
      rid: repo.rid,
      releaseId: release.id,
    });
    if (next) release = next;
  }

  const delegateDids = $derived(repo.delegates.map(d => d.did));

  function canEditMetadata(artifact: Artifact): boolean {
    return (
      isDelegateOrAuthor(
        config.publicKey,
        delegateDids,
        artifact.author.did,
      ) === true
    );
  }

  // Hide the Attest button for the artifact's own author — attestation
  // is a vouching signal from someone other than the author, and a
  // self-attestation is a no-op in the COB. Delegates who happen to also
  // be the author follow the same rule.
  function isOwnArtifact(artifact: Artifact): boolean {
    return publicKeyFromDid(artifact.author.did) === config.publicKey;
  }

  type PeerRole = "author" | "delegate" | "other";

  // Classify a peer DID relative to a specific artifact so the UI can
  // visually rank attestations / locations / redactions by trust. The
  // artifact author wins over delegate if both apply.
  function peerRole(did: string, artifact: Artifact): PeerRole {
    if (did === artifact.author.did) return "author";
    if (delegateDids.includes(did)) return "delegate";
    return "other";
  }

  // Redactions written by the artifact's author or a repo delegate are
  // treated as authoritative — we blur the artifact body and surface the
  // reason so users can still read why it was withdrawn.
  function trustedRedactions(artifact: Artifact) {
    return artifact.redactions.filter(
      r => peerRole(r.peer.did, artifact) !== "other",
    );
  }

  // Reveal toggles so the locations/attestations sections stay tucked
  // away by default but can be expanded per artifact.
  const revealLocations = $state<Record<string, boolean>>({});
  const revealAttestations = $state<Record<string, boolean>>({});
  // Per-artifact redact form: { reason } when open, undefined when closed.
  // Inline because window.prompt is a no-op in Tauri's WebKit webview, so
  // the dialog-based flow silently did nothing.
  const redactDraft = $state<Record<string, { reason: string } | undefined>>(
    {},
  );
  // Per-artifact toggle for the "add metadata" form. The form is hidden
  // by default since metadata edits are rare; users click "Add metadata"
  // to reveal it.
  const showMetadataForm = $state<Record<string, boolean>>({});

  const draft = $state<Record<string, { key: string; value: string }>>({});
  $effect(() => {
    for (const a of release.artifacts) {
      if (!draft[a.cid]) draft[a.cid] = { key: "", value: "" };
    }
  });

  const localShared = $state<Record<string, boolean>>({});
  const localAvailable = $state<Record<string, boolean>>({});
  const busy = $state<Record<string, boolean>>({});
  // Per-artifact error messages displayed inline under the action row.
  // Cleared when the user retries the same artifact's action.
  const actionErrors = $state<Record<string, string | undefined>>({});

  // Translate a backend error into something a user can act on. Falls
  // through to the raw message for codes we haven't styled yet.
  function describeError(err: unknown): string {
    if (err instanceof InvokeError) {
      if (err.code === "ArtifactError.CidMismatch") {
        return `${err.message}. Pick the original file that produced this CID.`;
      }
      if (err.code === "ArtifactError.NoLocations") {
        return "This artifact has no advertised locations to fetch from.";
      }
      if (err.code === "ArtifactError.NoReachableLocations") {
        return "No iroh providers are currently reachable for this artifact.";
      }
      if (err.code === "ArtifactError.AllTransportsFailed") {
        return err.message;
      }
      return err.message;
    }
    return String(err);
  }
  const progress = $state<
    Record<string, { stage: string; bytes?: number } | undefined>
  >({});

  type ProgressEvent = { cid: string; stage: string; bytes?: number };
  let unlistenProgress: (() => void) | undefined;

  onMount(async () => {
    // Progress events and local-availability checks both require the running
    // node, which only exists under the Tauri runtime. Skip them otherwise
    // (e.g. the HTTP test driver) so `listen` doesn't hit a missing IPC layer.
    if (!window.__TAURI_INTERNALS__) {
      return;
    }
    unlistenProgress = await listen<ProgressEvent>("artifact_progress", e => {
      const { cid, stage, bytes } = e.payload;
      if (stage === "done") {
        progress[cid] = undefined;
        // Refresh the local-availability pill — a finished download means
        // the bytes are now in the store even if we never seeded.
        void refreshAvailability(cid);
      } else {
        progress[cid] = { stage, bytes };
      }
    });
    for (const a of release.artifacts) {
      void refreshAvailability(a.cid);
    }
  });

  onDestroy(() => {
    if (unlistenProgress) unlistenProgress();
  });

  async function refreshAvailability(cid: string) {
    try {
      localAvailable[cid] = await invoke<boolean>("is_seeding", {
        rid: repo.rid,
        cid,
      });
    } catch {
      localAvailable[cid] = false;
    }
  }

  // Strict — only reflect *this* device's state. Falling back to the
  // DTO's flag used to flicker the wrong answer for the "seeded on
  // another machine" case, where the COB says we shared an iroh URL
  // but the bytes aren't in this process's store.
  function isShared(a: Artifact): boolean {
    return localShared[a.cid] ?? a.seededFromHere;
  }

  function isAvailableLocally(a: Artifact): boolean {
    return localAvailable[a.cid] === true;
  }

  // The DTO groups URLs by peer (one Location per DID), so
  // `locations.length` undercounts when a peer advertises multiple URLs
  // (e.g. iroh:// + https://). Count distinct URLs across all peers —
  // if two peers advertise the same URL, that's still one location.
  function totalLocationUrls(a: Artifact): number {
    const seen = new Set<string>();
    for (const loc of a.locations) {
      for (const url of loc.urls) seen.add(url);
    }
    return seen.size;
  }

  async function seed(artifact: Artifact) {
    let path: string | null;
    if (artifact.kind === "collection") {
      path = await invoke<string | null>("pick_artifact_directory");
    } else {
      const files = await invoke<string[]>("pick_artifact_files");
      path = files[0] ?? null;
    }
    if (!path) return;
    busy[artifact.cid] = true;
    actionErrors[artifact.cid] = undefined;
    try {
      await invoke("seed_artifact", {
        rid: repo.rid,
        releaseId: release.id,
        cid: artifact.cid,
        sourcePath: path,
      });
      localShared[artifact.cid] = true;
      localAvailable[artifact.cid] = true;
    } catch (err) {
      console.error("seed failed", err);
      actionErrors[artifact.cid] = describeError(err);
    } finally {
      busy[artifact.cid] = false;
    }
  }

  async function unseed(artifact: Artifact) {
    busy[artifact.cid] = true;
    actionErrors[artifact.cid] = undefined;
    try {
      await invoke("unseed_artifact", {
        rid: repo.rid,
        releaseId: release.id,
        cid: artifact.cid,
      });
      localShared[artifact.cid] = false;
      void refreshAvailability(artifact.cid);
    } catch (err) {
      console.error("unseed failed", err);
      actionErrors[artifact.cid] = describeError(err);
    } finally {
      busy[artifact.cid] = false;
    }
  }

  async function download(artifact: Artifact) {
    const isCollection = artifact.kind === "collection";
    const dest = isCollection
      ? await invoke<string | null>("pick_artifact_directory")
      : await invoke<string | null>("pick_artifact_save_path", {
          suggestedName: artifact.name,
        });
    if (!dest) return;

    busy[artifact.cid] = true;
    actionErrors[artifact.cid] = undefined;
    try {
      await invoke("download_artifact", {
        rid: repo.rid,
        releaseId: release.id,
        cid: artifact.cid,
        dest,
        seed: get(autoSeedArtifacts),
      });
      // The node seeds during the fetch when the user opted in, so refresh
      // to pick up the new `iroh://` location and pill state.
      await refresh();
      void refreshAvailability(artifact.cid);
    } catch (err) {
      console.error("download failed", err);
      actionErrors[artifact.cid] = describeError(err);
    } finally {
      busy[artifact.cid] = false;
    }
  }

  async function attest(artifact: Artifact) {
    busy[artifact.cid] = true;
    actionErrors[artifact.cid] = undefined;
    try {
      await invoke("attest_artifact", {
        rid: repo.rid,
        releaseId: release.id,
        cid: artifact.cid,
      });
      await refresh();
    } catch (err) {
      console.error("attest failed", err);
      actionErrors[artifact.cid] = describeError(err);
    } finally {
      busy[artifact.cid] = false;
    }
  }

  function openRedact(artifact: Artifact) {
    redactDraft[artifact.cid] = { reason: "" };
  }

  function cancelRedact(artifact: Artifact) {
    redactDraft[artifact.cid] = undefined;
  }

  async function confirmRedact(artifact: Artifact) {
    const draft = redactDraft[artifact.cid];
    if (!draft) return;
    const reason = draft.reason.trim();
    if (!reason) {
      actionErrors[artifact.cid] = "Please provide a reason for the redaction.";
      return;
    }
    busy[artifact.cid] = true;
    actionErrors[artifact.cid] = undefined;
    try {
      await invoke("redact_artifact", {
        rid: repo.rid,
        releaseId: release.id,
        cid: artifact.cid,
        reason,
      });
      redactDraft[artifact.cid] = undefined;
      await refresh();
    } catch (err) {
      console.error("redact failed", err);
      actionErrors[artifact.cid] = describeError(err);
    } finally {
      busy[artifact.cid] = false;
    }
  }

  async function setMetadata(artifact: Artifact, key: string, raw: string) {
    if (!key.trim()) return;
    let value: unknown;
    try {
      value = JSON.parse(raw);
    } catch {
      value = raw;
    }
    busy[artifact.cid] = true;
    try {
      await invoke("set_metadata", {
        rid: repo.rid,
        releaseId: release.id,
        cid: artifact.cid,
        key,
        value,
      });
      draft[artifact.cid] = { key: "", value: "" };
      showMetadataForm[artifact.cid] = false;
      await refresh();
    } catch (err) {
      console.error("set_metadata failed", err);
    } finally {
      busy[artifact.cid] = false;
    }
  }

  async function removeMetadata(artifact: Artifact, key: string) {
    busy[artifact.cid] = true;
    try {
      await invoke("remove_metadata", {
        rid: repo.rid,
        releaseId: release.id,
        cid: artifact.cid,
        key,
      });
      await refresh();
    } catch (err) {
      console.error("remove_metadata failed", err);
    } finally {
      busy[artifact.cid] = false;
    }
  }

  function progressText(cid: string): string {
    const p = progress[cid];
    if (!p) return "";
    if (p.stage === "downloading" && p.bytes !== undefined) {
      const mb = (p.bytes / (1024 * 1024)).toFixed(1);
      return `downloading ${mb} MiB`;
    }
    return p.stage;
  }
</script>

<style>
  .page {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  .breadcrumb {
    display: flex;
    align-items: center;
    gap: 0.375rem;
  }
  .breadcrumb-link {
    cursor: pointer;
    background: none;
    border: none;
    padding: 0;
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
  }
  .breadcrumb-link:hover {
    color: var(--color-text-primary);
  }
  .header {
    padding: 1.25rem 1.5rem;
    border-bottom: 1px solid var(--color-border-subtle);
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  .header h1 {
    margin: 0;
    font: var(--txt-body-l-semibold);
    word-break: break-word;
  }
  .header .meta {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
  }
  .commit-panel {
    background-color: var(--color-surface-canvas);
    border-radius: var(--border-radius-sm);
    padding: 0.75rem 1rem;
    font: var(--txt-body-m-regular);
  }
  .commit-summary {
    font: var(--txt-body-m-semibold);
    color: var(--color-text-primary);
    margin-bottom: 0.25rem;
  }
  .commit-body {
    white-space: pre-wrap;
    color: var(--color-text-secondary);
    font: var(--txt-body-s-mono);
  }
  .artifact {
    padding: 1rem 1.5rem;
    border-bottom: 1px solid var(--color-border-subtle);
    background-color: var(--color-surface-1);
  }
  .artifact-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex-wrap: wrap;
  }
  .name {
    font: var(--txt-body-m-semibold);
  }
  .kind {
    font: var(--txt-body-s-regular);
    color: var(--color-text-secondary);
  }
  .pill {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    height: 1.5rem;
    padding: 0 0.5rem;
    border-radius: var(--border-radius-sm);
    border: 1px solid var(--color-border-subtle);
    font: var(--txt-body-s-regular);
    color: var(--color-text-secondary);
  }
  .pill.available {
    background-color: var(--color-feedback-success-bg);
    border-color: var(--color-feedback-success-border);
    color: var(--color-feedback-success-text);
  }
  .pill.other-device {
    background-color: var(--color-feedback-warning-bg);
    border-color: var(--color-feedback-warning-border);
    color: var(--color-feedback-warning-text);
  }
  .actions {
    margin-left: auto;
    display: flex;
    gap: 0.25rem;
  }
  .actions button {
    padding: 0.25rem 0.5rem;
    background-color: var(--color-surface-subtle);
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    cursor: pointer;
    font: var(--txt-body-s-regular);
  }
  .actions button.danger {
    color: var(--color-fill-error);
  }
  .actions button:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .meta-line {
    display: flex;
    gap: 1rem;
    margin-top: 0.5rem;
    font: var(--txt-body-s-regular);
    color: var(--color-text-secondary);
  }
  .metadata {
    display: grid;
    grid-template-columns: auto 1fr;
    column-gap: 0.5rem;
    row-gap: 0.125rem;
    margin: 0.5rem 0 0;
    font: var(--txt-body-s-regular);
  }
  .metadata dt {
    font-family: var(--font-family-mono);
    color: var(--color-text-secondary);
  }
  .metadata dd {
    margin: 0;
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }
  .metadata .value {
    word-break: break-word;
  }
  .remove-meta {
    padding: 0 0.25rem;
    background: transparent;
    border: none;
    color: var(--color-text-secondary);
    cursor: pointer;
    line-height: 1;
  }
  .remove-meta:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .add-meta {
    display: flex;
    gap: 0.25rem;
    margin-top: 0.5rem;
  }
  .add-meta input {
    padding: 0.25rem 0.5rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    font: var(--txt-body-s-regular);
  }
  .add-meta input:first-child {
    font-family: var(--font-family-mono);
    width: 8rem;
  }
  .add-meta input:nth-child(2) {
    flex: 1;
  }
  .add-meta button {
    padding: 0.25rem 0.5rem;
    background-color: var(--color-surface-subtle);
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    cursor: pointer;
    font: var(--txt-body-s-regular);
  }
  .add-meta button:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .progress {
    margin-top: 0.5rem;
    font: var(--txt-body-s-regular);
    color: var(--color-text-secondary);
  }
  .action-error {
    margin-top: 0.5rem;
    padding: 0.5rem 0.75rem;
    background-color: var(--color-feedback-error-bg);
    border: 1px solid var(--color-feedback-error-border);
    border-radius: var(--border-radius-sm);
    color: var(--color-feedback-error-text);
    font: var(--txt-body-s-regular);
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
  }
  .action-error .dismiss {
    margin-left: auto;
    background: none;
    border: none;
    cursor: pointer;
    color: inherit;
    line-height: 1;
    font-size: 1rem;
    padding: 0;
  }
  .redact-form {
    margin-top: 0.5rem;
    padding: 0.75rem;
    border: 1px solid var(--color-feedback-error-border);
    border-radius: var(--border-radius-sm);
    background-color: var(--color-feedback-error-bg);
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .redact-form-title {
    font: var(--txt-body-s-semibold);
    color: var(--color-feedback-error-text);
  }
  .redact-form textarea {
    resize: vertical;
    min-height: 3rem;
    padding: 0.4rem 0.5rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    background-color: var(--color-surface-canvas);
    font: var(--txt-body-s-regular);
  }
  .redact-form .row {
    display: flex;
    gap: 0.5rem;
    justify-content: flex-end;
  }
  .redact-form button {
    padding: 0.25rem 0.75rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    background-color: var(--color-surface-subtle);
    cursor: pointer;
    font: var(--txt-body-s-regular);
  }
  .redact-form button.confirm {
    background-color: var(--color-feedback-error-fill);
    color: var(--color-text-inverse);
    border-color: var(--color-feedback-error-fill);
  }
  .redact-form button:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .empty {
    padding: 2rem;
    color: var(--color-text-secondary);
    text-align: center;
  }
  .reveal {
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    font: var(--txt-body-s-regular);
    color: var(--color-text-secondary);
    text-decoration: underline dotted;
  }
  .reveal:hover {
    color: var(--color-text-primary);
  }
  .peer-list {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
    margin-top: 0.5rem;
    padding: 0.5rem 0.75rem;
    background-color: var(--color-surface-canvas);
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
  }
  .peer-row {
    display: flex;
    align-items: flex-start;
    flex-wrap: wrap;
    gap: 0.5rem;
    font: var(--txt-body-s-regular);
  }
  .role-badge {
    display: inline-flex;
    align-items: center;
    height: 1.25rem;
    padding: 0 0.375rem;
    border-radius: var(--border-radius-sm);
    font: var(--txt-body-s-regular);
    border: 1px solid var(--color-border-subtle);
    color: var(--color-text-secondary);
  }
  .role-badge.author {
    background-color: var(--color-feedback-success-bg);
    border-color: var(--color-feedback-success-border);
    color: var(--color-feedback-success-text);
  }
  .role-badge.delegate {
    background-color: var(--color-feedback-warning-bg);
    border-color: var(--color-feedback-warning-border);
    color: var(--color-feedback-warning-text);
  }
  .peer-url-list {
    margin: 0;
    padding-left: 1rem;
    width: 100%;
    color: var(--color-text-secondary);
    word-break: break-all;
  }
  .reason {
    color: var(--color-text-primary);
    margin-left: 0.25rem;
  }
  .redactions {
    margin-top: 0.5rem;
    padding: 0.5rem 0.75rem;
    background-color: var(--color-feedback-error-bg);
    border: 1px solid var(--color-feedback-error-border);
    border-radius: var(--border-radius-sm);
    color: var(--color-feedback-error-text);
  }
  .redactions-title {
    font: var(--txt-body-s-semibold);
    margin-bottom: 0.25rem;
  }
  .redaction-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.375rem;
    font: var(--txt-body-s-regular);
  }
  .blur {
    filter: blur(4px);
    pointer-events: none;
    user-select: none;
    opacity: 0.6;
  }
</style>

<Layout>
  <div class="page">
    <Topbar>
      <div class="breadcrumb">
        <Icon name="commit" />
        <button
          class="breadcrumb-link"
          onclick={() =>
            router.push({
              resource: "repo.releases",
              rid: repo.rid,
              filter,
            })}>
          {filter === "delegate" ? "Delegate releases" : "All releases"}
        </button>
        <Icon name="chevron-right" />
        <Id id={release.id} clipboard={release.id} placement="bottom-start" />
      </div>
    </Topbar>

    <ScrollArea style="flex: 1; min-height: 0;">
      <div class="header">
        <h1>
          {release.tagName ??
            release.commitSummary ??
            `Release ${release.oid.slice(0, 7)}`}
        </h1>
        <div class="meta">
          <NodeId {...authorForNodeId(release.creator)} />
          released
          <Id id={release.id} clipboard={release.id} ariaLabel="Release ID" />
          from commit
          <Id id={release.oid} clipboard={release.oid} ariaLabel="Commit OID" />
          {#if release.tagName}
            <span class="pill">{release.tagName}</span>
          {/if}
          <span title={absoluteTimestamp(release.timestamp * 1000)}>
            {formatTimestamp(release.timestamp * 1000)}
          </span>
        </div>
        {#if commit}
          <div class="commit-panel">
            <div class="commit-summary">{commit.summary}</div>
            {#if commit.message.trim() !== commit.summary.trim()}
              <div class="commit-body">
                {(commit.message.startsWith(commit.summary)
                  ? commit.message.slice(commit.summary.length)
                  : commit.message
                ).trim()}
              </div>
            {/if}
          </div>
        {/if}
      </div>

      {#each release.artifacts as artifact (artifact.cid)}
        {@const trusted = trustedRedactions(artifact)}
        <div class="artifact">
          {#if trusted.length > 0}
            <div class="redactions">
              <div class="redactions-title">
                Redacted by {trusted.length === 1
                  ? "the artifact author or a delegate"
                  : "delegates / the artifact author"}
              </div>
              {#each trusted as redaction (redaction.peer.did)}
                <div class="redaction-row">
                  <NodeId {...authorForNodeId(redaction.peer)} />
                  <span
                    class="role-badge {peerRole(redaction.peer.did, artifact)}">
                    {peerRole(redaction.peer.did, artifact)}
                  </span>
                  <span class="reason">{redaction.reason}</span>
                </div>
              {/each}
            </div>
          {/if}
          <div class:blur={trusted.length > 0}>
            <div class="artifact-row">
              <span class="name">{artifact.name}</span>
              <span class="kind">[{artifact.kind}]</span>
              {#if isAvailableLocally(artifact)}
                <span class="pill available" title="Available in local store">
                  <Icon name="checkmark" />
                  Local
                </span>
              {/if}
              {#if !isAvailableLocally(artifact) && artifact.seededFromOther}
                <span
                  class="pill other-device"
                  title="You advertised this artifact via iroh from a different device (e.g. the CLI). The bytes aren't in this app's store — that peer must be online to serve them.">
                  <Icon name="device" />
                  Other device
                </span>
              {/if}
              <div class="actions">
                <button
                  onclick={() => download(artifact)}
                  disabled={busy[artifact.cid] ||
                    !$artifactNodeRunning ||
                    (!isAvailableLocally(artifact) &&
                      totalLocationUrls(artifact) === 0)}
                  title={!$artifactNodeRunning
                    ? "The artifact node is not running"
                    : isAvailableLocally(artifact)
                      ? "Export the locally-stored copy to disk"
                      : totalLocationUrls(artifact) === 0
                        ? "No locations to download from"
                        : "Fetch from a peer and write to disk"}>
                  {isAvailableLocally(artifact) ? "Save to disk" : "Download"}
                </button>
                {#if isShared(artifact)}
                  <button
                    onclick={() => unseed(artifact)}
                    disabled={busy[artifact.cid] || !$artifactNodeRunning}
                    title={$artifactNodeRunning
                      ? undefined
                      : "The artifact node is not running"}>
                    Unseed
                  </button>
                {:else}
                  <button
                    onclick={() => seed(artifact)}
                    disabled={busy[artifact.cid] || !$artifactNodeRunning}
                    title={$artifactNodeRunning
                      ? undefined
                      : "The artifact node is not running"}>
                    Seed
                  </button>
                {/if}
                {#if !isOwnArtifact(artifact)}
                  <button
                    onclick={() => attest(artifact)}
                    disabled={busy[artifact.cid]}>
                    Attest
                  </button>
                {/if}
                {#if canEditMetadata(artifact)}
                  <button
                    class="danger"
                    onclick={() => openRedact(artifact)}
                    disabled={busy[artifact.cid] ||
                      redactDraft[artifact.cid] !== undefined}>
                    Redact
                  </button>
                {/if}
              </div>
            </div>
            <div style:margin-top="0.375rem">
              <Id id={artifact.cid} clipboard={artifact.cid} shorten={false} />
            </div>
            <div class="meta-line">
              <button
                class="reveal"
                onclick={() =>
                  (revealAttestations[artifact.cid] =
                    !revealAttestations[artifact.cid])}
                disabled={artifact.attestations.length === 0}>
                {artifact.attestations.length} attestations
              </button>
              <button
                class="reveal"
                onclick={() =>
                  (revealLocations[artifact.cid] =
                    !revealLocations[artifact.cid])}
                disabled={totalLocationUrls(artifact) === 0}>
                {totalLocationUrls(artifact)} locations
              </button>
              {#if artifact.redactions.length > trusted.length}
                <span style:color="var(--color-feedback-error-text)">
                  {artifact.redactions.length - trusted.length} other redactions
                </span>
              {/if}
            </div>
            {#if revealAttestations[artifact.cid] && artifact.attestations.length > 0}
              <div class="peer-list">
                {#each artifact.attestations as att (att.did)}
                  <div class="peer-row">
                    <NodeId {...authorForNodeId(att)} />
                    {#if peerRole(att.did, artifact) !== "other"}
                      <span class="role-badge {peerRole(att.did, artifact)}">
                        {peerRole(att.did, artifact)}
                      </span>
                    {/if}
                  </div>
                {/each}
              </div>
            {/if}
            {#if revealLocations[artifact.cid] && artifact.locations.length > 0}
              <div class="peer-list">
                {#each artifact.locations as loc (loc.peer.did)}
                  <div class="peer-row">
                    <NodeId {...authorForNodeId(loc.peer)} />
                    {#if peerRole(loc.peer.did, artifact) !== "other"}
                      <span
                        class="role-badge {peerRole(loc.peer.did, artifact)}">
                        {peerRole(loc.peer.did, artifact)}
                      </span>
                    {/if}
                    <ul class="peer-url-list">
                      {#each loc.urls as url}
                        <li>{url}</li>
                      {/each}
                    </ul>
                  </div>
                {/each}
              </div>
            {/if}
            {#if artifact.redactions.length > trusted.length}
              <div class="peer-list">
                {#each artifact.redactions.filter(r => peerRole(r.peer.did, artifact) === "other") as redaction (redaction.peer.did)}
                  <div class="peer-row">
                    <NodeId {...authorForNodeId(redaction.peer)} />
                    <span class="reason">{redaction.reason}</span>
                  </div>
                {/each}
              </div>
            {/if}
            {#if redactDraft[artifact.cid]}
              <form
                class="redact-form"
                onsubmit={e => {
                  e.preventDefault();
                  void confirmRedact(artifact);
                }}>
                <div class="redact-form-title">
                  Redact "{artifact.name}"?
                </div>
                <div style:font="var(--txt-body-s-regular)">
                  This records a signed redaction on the release COB. Other
                  peers will see it and the artifact will appear blurred for
                  delegates and authors. The CID and existing locations stay on
                  the COB.
                </div>
                <textarea
                  placeholder="Reason (required)"
                  bind:value={redactDraft[artifact.cid]!.reason}
                  disabled={busy[artifact.cid]}>
                </textarea>
                <div class="row">
                  <button
                    type="button"
                    onclick={() => cancelRedact(artifact)}
                    disabled={busy[artifact.cid]}>
                    Cancel
                  </button>
                  <button
                    type="submit"
                    class="confirm"
                    disabled={busy[artifact.cid]}>
                    {busy[artifact.cid] ? "Redacting…" : "Redact"}
                  </button>
                </div>
              </form>
            {/if}
            {#if artifact.metadata && Object.keys(artifact.metadata).length > 0}
              <dl class="metadata">
                {#each Object.entries(artifact.metadata) as [key, value] (key)}
                  <dt>{key}</dt>
                  <dd>
                    <span class="value">
                      {typeof value === "string"
                        ? value
                        : JSON.stringify(value)}
                    </span>
                    {#if canEditMetadata(artifact)}
                      <button
                        class="remove-meta"
                        title="Remove"
                        onclick={() => removeMetadata(artifact, key)}
                        disabled={busy[artifact.cid]}>
                        ×
                      </button>
                    {/if}
                  </dd>
                {/each}
              </dl>
            {/if}
            {#if canEditMetadata(artifact) && draft[artifact.cid]}
              {#if showMetadataForm[artifact.cid]}
                <form
                  class="add-meta"
                  onsubmit={e => {
                    e.preventDefault();
                    const d = draft[artifact.cid];
                    void setMetadata(artifact, d.key, d.value);
                  }}>
                  <input
                    type="text"
                    placeholder="key"
                    bind:value={draft[artifact.cid].key}
                    disabled={busy[artifact.cid]} />
                  <input
                    type="text"
                    placeholder="value (string or JSON)"
                    bind:value={draft[artifact.cid].value}
                    disabled={busy[artifact.cid]} />
                  <button type="submit" disabled={busy[artifact.cid]}>
                    Add
                  </button>
                  <button
                    type="button"
                    onclick={() => (showMetadataForm[artifact.cid] = false)}
                    disabled={busy[artifact.cid]}>
                    Cancel
                  </button>
                </form>
              {:else}
                <button
                  class="reveal"
                  style:margin-top="0.5rem"
                  onclick={() => (showMetadataForm[artifact.cid] = true)}>
                  + Add metadata
                </button>
              {/if}
            {/if}
            {#if progress[artifact.cid]}
              <div class="progress">{progressText(artifact.cid)}</div>
            {/if}
            {#if actionErrors[artifact.cid]}
              <div class="action-error" role="alert">
                <Icon name="warning" />
                <span>{actionErrors[artifact.cid]}</span>
                <button
                  class="dismiss"
                  title="Dismiss"
                  onclick={() => (actionErrors[artifact.cid] = undefined)}>
                  ×
                </button>
              </div>
            {/if}
          </div>
        </div>
      {:else}
        <div class="empty">No artifacts in this release</div>
      {/each}
    </ScrollArea>
  </div>
</Layout>
