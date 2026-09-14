<script lang="ts">
  import type { Author } from "@bindings/cob/Author";
  import type { Artifact } from "@bindings/cob/release/Artifact";
  import type { ArtifactDigest } from "@bindings/cob/release/ArtifactDigest";
  import type { Release } from "@bindings/cob/release/Release";
  import type { ReleaseScope } from "@bindings/cob/release/ReleaseScope";
  import type { Config } from "@bindings/config/Config";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import { slide } from "svelte/transition";

  import { pickFiles, pickFolder, pickLike } from "@app/lib/artifactPickers";
  import { basename } from "@app/lib/embeds";
  import { artifactNodeRunning } from "@app/lib/events";
  import { invoke, InvokeError } from "@app/lib/invoke";
  import { show } from "@app/lib/modal";
  import {
    artifactView,
    attestedBy,
    canAttest,
    canEditMetadata,
    delegatesFirst,
    displayMetadataValue,
    locationsByNode,
    parseMetadataValue,
    redactedByDelegate,
  } from "@app/lib/releases";
  import * as router from "@app/lib/router";
  import type { Picked, StagedArtifact } from "@app/lib/stageArtifacts";
  import { effective, stageArtifacts } from "@app/lib/stageArtifacts";
  import {
    authorForNodeId,
    didFromPublicKey,
    formatBytes,
    shortenCids,
  } from "@app/lib/utils";

  import ArtifactDownloadButton from "@app/components/ArtifactDownloadButton.svelte";
  import Button from "@app/components/Button.svelte";
  import DelegateBadge from "@app/components/DelegateBadge.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import InlineTitle from "@app/components/InlineTitle.svelte";
  import NodeId from "@app/components/NodeId.svelte";
  import Popover, { closeFocused } from "@app/components/Popover.svelte";
  import ReleaseMetadata from "@app/components/ReleaseMetadata.svelte";
  import ScrollArea from "@app/components/ScrollArea.svelte";
  import ShareButton from "@app/components/ShareButton.svelte";
  import TextInput from "@app/components/TextInput.svelte";
  import Topbar from "@app/components/Topbar.svelte";
  import YouBadge from "@app/components/YouBadge.svelte";
  import ConfirmDeleteRelease from "@app/modals/ConfirmDeleteRelease.svelte";
  import ConfirmRedact from "@app/modals/ConfirmRedact.svelte";
  import ConfirmRegisterArtifacts from "@app/modals/ConfirmRegisterArtifacts.svelte";

  import Layout from "./Layout.svelte";

  interface Props {
    repo: RepoInfo;
    config: Config;
    release: Release;
    scope?: ReleaseScope;
    artifactScope?: ReleaseScope;
  }

  /* eslint-disable prefer-const */
  let { repo, config, release, scope, artifactScope }: Props = $props();
  /* eslint-enable prefer-const */

  const SIZE_KEY = "sizeBytes";
  // An artifact COB carries no display name of its own. `title` in the metadata
  // map is the convention for one, set by the artifact's author or a delegate,
  // with the file name always present underneath as the fallback. Reading it
  // from the COB rather than guessing from the file name keeps every client
  // showing the same thing.
  const TITLE_KEY = "title";

  const ownDid = $derived(didFromPublicKey(config.publicKey));

  // A release COB carries no name of its own. The backend resolves one from the
  // annotated tag's message, falling back to the commit subject; failing both,
  // the tag name and then the release id stand in.
  const title = $derived(release.title || release.tagName || release.id);
  const delegateIds = $derived(new Set(repo.delegates.map(d => d.did)));

  // An artifact redacted by its own author or a delegate is hidden by default.
  let showRedacted = $state(false);

  const view = $derived(
    artifactView(
      release.artifacts,
      delegateIds,
      artifactScope ?? "trusted",
      showRedacted,
    ),
  );

  function artifactSize(artifact: Artifact): string | undefined {
    const size = artifact.metadata[SIZE_KEY];
    return typeof size === "number" ? formatBytes(size) : undefined;
  }

  // Metadata entries other than the size hint, shown as raw key/value pairs.
  function otherMetadata(artifact: Artifact): [string, unknown][] {
    return Object.entries(artifact.metadata).filter(
      ([key]) => key !== SIZE_KEY,
    );
  }

  function attestationLabel(attestations: Author[]): string | undefined {
    const count = attestedBy(attestations, delegateIds);
    if (count === 0) {
      return undefined;
    }
    return `Attested by ${count} delegate${count === 1 ? "" : "s"}`;
  }

  // Computed across the artifacts actually on screen, so the shortest form
  // that tells them apart is the one shown.
  const cidLabels = $derived(shortenCids(view.shown.map(a => a.cid)));

  function artifactTitle(artifact: Artifact): string | undefined {
    const title = artifact.metadata[TITLE_KEY];
    return typeof title === "string" && title.trim() !== ""
      ? title.trim()
      : undefined;
  }

  // Which artifact cards are expanded, keyed by CID.
  const expanded: Record<string, boolean> = $state({});

  function toggleExpanded(cid: string) {
    expanded[cid] = !expanded[cid];
  }

  let registering = $state(false);
  let registerError: string | undefined = $state();

  // Registering an artifact is open to any user, so this is not gated on
  // authorship. Seeding is what makes the bytes reachable, and it needs the
  // artifact node, so it is attempted only when the node answers: without it
  // the artifact is registered but nobody seeds it.
  // Hash and measure first, then ask. Registering signs a COB entry that syncs
  // to everyone holding the repo and, with the node up, starts seeding the
  // bytes; none of that can be withdrawn afterwards, so the last chance to
  // change your mind has to come before it, with the sizes and file counts on
  // screen. Picking a folder by mistake is the case worth catching.
  async function registerArtifacts(paths: string[]) {
    if (paths.length === 0) {
      return;
    }
    registering = true;
    registerError = undefined;

    let nodeRunning: boolean;
    try {
      nodeRunning = await invoke<boolean>("artifact_node_running");
    } catch {
      nodeRunning = false;
    }

    let picked: Picked[];
    try {
      [picked] = await Promise.all([
        Promise.all(
          paths.map(async path => ({
            path,
            name: basename(path),
            digest: await invoke<ArtifactDigest>("compute_artifact_cid", {
              path,
            }),
          })),
        ),
        refreshSeeded(),
      ]);
    } catch (error) {
      console.error("Reading the selection failed", error);
      registerError = "Could not read what you picked.";
      return;
    } finally {
      registering = false;
    }

    show({
      component: ConfirmRegisterArtifacts,
      props: {
        staged: stageArtifacts(picked, release.artifacts, {
          ownDid,
          delegates: delegateIds,
          nodeRunning,
          seeded: seededCids,
        }),
        confirm: registerStaged,
      },
    });
  }

  async function registerStaged(
    staged: StagedArtifact[],
    includeRedacted: boolean,
  ) {
    registering = true;
    registerError = undefined;

    let unseeded = 0;
    let seedFailures = 0;
    try {
      // One at a time: each is its own signed COB entry, so a failure part way
      // through leaves the earlier ones registered.
      for (const item of staged) {
        const { register, seed } = effective(item, includeRedacted);
        if (register) {
          await invoke("register_artifact", {
            rid: repo.rid,
            releaseId: release.id,
            cid: item.digest.cid,
            name: item.name,
            sizeBytes: item.digest.sizeBytes,
          });
        }
        if (seed) {
          try {
            await invoke("seed_artifact", {
              rid: repo.rid,
              releaseId: release.id,
              cid: item.digest.cid,
              sourcePath: item.path,
            });
          } catch {
            seedFailures += 1;
          }
        } else if (register && !item.existing) {
          unseeded += 1;
        }
      }
      if (unseeded > 0) {
        registerError =
          "Registered, but your artifact node is not running, so they cannot be downloaded yet.";
      } else if (seedFailures > 0) {
        registerError = `Registered, but ${seedFailures} could not be seeded and cannot be downloaded yet.`;
      }
    } catch (error) {
      console.error("Registering artifacts failed", error);
      registerError = "Registering failed.";
    } finally {
      registering = false;
      await reload();
      await refreshSeeded();
    }
  }

  async function chooseFiles() {
    closeFocused();
    const paths = await pickFiles();
    if (paths.length > 0) {
      await registerArtifacts(paths);
    }
  }

  async function chooseFolder() {
    closeFocused();
    const path = await pickFolder();
    if (path) {
      await registerArtifacts([path]);
    }
  }

  function openDelete() {
    show({
      component: ConfirmDeleteRelease,
      props: {
        title,
        shared: release.artifacts.some(a => a.author.did !== ownDid),
        confirm: async () => {
          await invoke("delete_release", {
            rid: repo.rid,
            releaseId: release.id,
          });
          await router.push({ resource: "repo.releases", rid: repo.rid });
        },
      },
    });
  }

  // Seeding needs the artifact node, so the seed action is only offered while
  // it answers.
  const artifactNodeUp = $derived($artifactNodeRunning);

  function openRedact(artifact: Artifact) {
    show({
      component: ConfirmRedact,
      props: {
        name: artifactTitle(artifact) ?? artifact.name,
        seeding: seededCids.has(artifact.cid),
        confirm: async (reason: string) => {
          await invoke("redact_artifact", {
            rid: repo.rid,
            releaseId: release.id,
            cid: artifact.cid,
            reason,
          });
          await reload();
          await refreshSeeded();
        },
      },
    });
  }

  // Writing metadata is constrained to the artifact's author or a repository
  // delegate, so the editor stays hidden for everyone else.
  // The key being edited, as `<cid>\n<key>`, so two artifacts sharing a key
  // name never open each other's editor. An empty key means a new entry.
  let editing: string | undefined = $state();
  let draftKey = $state("");
  let draftValue = $state("");
  let saving = $state(false);
  let metadataError: string | undefined = $state();

  function editorId(cid: string, key: string): string {
    return `${cid}\n${key}`;
  }

  function startEdit(cid: string, key: string, value: unknown) {
    editing = editorId(cid, key);
    draftKey = key;
    draftValue = key === "" ? "" : displayMetadataValue(value);
    metadataError = undefined;
  }

  function cancelEdit() {
    editing = undefined;
    draftKey = "";
    draftValue = "";
    metadataError = undefined;
  }

  // Seeding lives in the node, not the COB, so it has to be asked for rather
  // than read off the release. One call per view, matched against the rows.
  let seededCids = $state<Set<string>>(new Set());

  async function refreshSeeded() {
    try {
      const cids = await invoke<string[]>("seeded_artifacts", {
        rid: repo.rid,
        releaseId: release.id,
      });
      seededCids = new Set(cids);
    } catch {
      // A node that is down seeds nothing we can confirm, so claim nothing.
      seededCids = new Set();
    }
  }

  // Attesting means reproducing the artifact: the backend hashes a local build
  // and signs only when it arrives at the same CID.
  let attesting: string | undefined = $state();
  let attestError: { cid: string; message: string } | undefined = $state();

  async function attest(artifact: Artifact) {
    const cid = artifact.cid;
    const path = await pickLike(artifact.directory);
    if (!path) {
      return;
    }

    attesting = cid;
    attestError = undefined;
    try {
      await invoke("attest_artifact", {
        rid: repo.rid,
        releaseId: release.id,
        cid,
        path,
      });
    } catch (error) {
      attestError = {
        cid,
        message:
          error instanceof InvokeError &&
          error.code === "ArtifactError.CidMismatch"
            ? "Your build has a different CID, so it was not attested."
            : "Attesting failed.",
      };
    } finally {
      attesting = undefined;
      await reload();
    }
  }

  let reseeding: string | undefined = $state();
  let reseedError: { cid: string; message: string } | undefined = $state();

  async function reseed(artifact: Artifact) {
    const cid = artifact.cid;
    const path = await pickLike(artifact.directory);
    if (!path) {
      return;
    }

    reseeding = cid;
    reseedError = undefined;
    try {
      const digest = await invoke<ArtifactDigest>("compute_artifact_cid", {
        path,
      });
      if (digest.cid !== cid) {
        reseedError = {
          cid,
          message: "This file has a different CID, so it was not seeded.",
        };
        return;
      }
      await invoke("seed_artifact", {
        rid: repo.rid,
        releaseId: release.id,
        cid,
        sourcePath: path,
      });
    } catch (error) {
      console.error("Seeding failed", error);
      reseedError = {
        cid,
        message: (await invoke<boolean>("artifact_node_running").catch(
          () => false,
        ))
          ? "Seeding failed."
          : "Your artifact node is not running.",
      };
    } finally {
      reseeding = undefined;
      await refreshSeeded();
      await reload();
    }
  }

  // CIDs with a stop request in flight, and the last one that failed.
  let unseeding = $state<Set<string>>(new Set());
  let unseedFailed: string | undefined = $state();

  async function stopSeeding(cid: string) {
    unseeding = new Set([...unseeding, cid]);
    unseedFailed = undefined;
    try {
      await invoke("unseed_artifact", {
        rid: repo.rid,
        releaseId: release.id,
        cid,
      });
    } catch (error) {
      console.error("Unseeding failed", error);
      unseedFailed = cid;
    } finally {
      unseeding = new Set([...unseeding].filter(c => c !== cid));
      await refreshSeeded();
      await reload();
    }
  }

  $effect(() => {
    // Re-ask when the route lands on another release.
    void release.id;
    void refreshSeeded();
  });

  // Callers reload after an action has already succeeded or failed, so a
  // failed refresh must not be reported as the action's outcome.
  async function reload() {
    try {
      const updated = await invoke<Release | null>("release_by_id", {
        rid: repo.rid,
        id: release.id,
      });
      if (updated) {
        release = updated;
      }
    } catch (error) {
      console.error("Refreshing the release failed", error);
    }
  }

  // Anyone may add a location, but only withdraw the ones they added: the COB
  // keys each location by the node that signed it.
  let addingLocation: string | undefined = $state();
  let draftUrl = $state("");
  let savingLocation = $state(false);
  let locationError: { cid: string; message: string } | undefined = $state();

  function startAddLocation(cid: string) {
    addingLocation = cid;
    draftUrl = "";
    locationError = undefined;
  }

  function cancelAddLocation() {
    addingLocation = undefined;
    draftUrl = "";
    locationError = undefined;
  }

  async function addLocation(cid: string) {
    const url = draftUrl.trim();
    if (url === "") {
      locationError = { cid, message: "A URL is required." };
      return;
    }

    savingLocation = true;
    locationError = undefined;
    try {
      await invoke("add_artifact_location", {
        rid: repo.rid,
        releaseId: release.id,
        cid,
        url,
      });
      cancelAddLocation();
      await reload();
    } catch (error) {
      console.error("Adding a location failed", error);
      locationError = {
        cid,
        message:
          error instanceof InvokeError &&
          error.code === "ArtifactError.InvalidLocation"
            ? "This is not a URL peers can fetch from."
            : "Adding the location failed.",
      };
    } finally {
      savingLocation = false;
    }
  }

  async function removeLocation(cid: string, url: string) {
    savingLocation = true;
    locationError = undefined;
    try {
      await invoke("remove_artifact_location", {
        rid: repo.rid,
        releaseId: release.id,
        cid,
        url,
      });
      closeFocused();
      await reload();
    } catch (error) {
      console.error("Removing a location failed", error);
      locationError = { cid, message: "Removing the location failed." };
    } finally {
      savingLocation = false;
    }
  }

  async function saveMetadata(artifact: Artifact, previousKey: string) {
    const key = draftKey.trim();
    if (key === "") {
      metadataError = "A key is required.";
      return;
    }
    if (key === SIZE_KEY) {
      metadataError = `"${SIZE_KEY}" is maintained by the app.`;
      return;
    }

    saving = true;
    metadataError = undefined;
    try {
      // Renaming a key is a remove plus a set, since the COB has no rename.
      if (previousKey !== "" && previousKey !== key) {
        await invoke("remove_artifact_metadata", {
          rid: repo.rid,
          releaseId: release.id,
          cid: artifact.cid,
          key: previousKey,
        });
      }
      await invoke("set_artifact_metadata", {
        rid: repo.rid,
        releaseId: release.id,
        cid: artifact.cid,
        key,
        value: parseMetadataValue(draftValue),
      });
      cancelEdit();
      await reload();
    } catch (error) {
      console.error("Saving artifact metadata failed", error);
      metadataError = "Saving failed.";
    } finally {
      saving = false;
    }
  }

  async function removeMetadata(artifact: Artifact, key: string) {
    saving = true;
    metadataError = undefined;
    try {
      await invoke("remove_artifact_metadata", {
        rid: repo.rid,
        releaseId: release.id,
        cid: artifact.cid,
        key,
      });
      cancelEdit();
      await reload();
    } catch (error) {
      console.error("Removing artifact metadata failed", error);
      metadataError = "Removing failed.";
    } finally {
      saving = false;
    }
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
    min-width: 0;
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
  .main {
    padding: 1.5rem 6rem;
    min-width: 0;
    max-width: 80rem;
    margin: 0 auto;
  }
  .title {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 0.75rem;
  }
  .title-chip {
    padding: 0;
    height: 2rem;
    width: 2rem;
    flex-shrink: 0;
    color: var(--color-text-tertiary);
  }
  .metadata-row {
    margin-bottom: 1.5rem;
  }
  .filter {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
    margin-bottom: 1rem;
  }
  .redacted-toggle {
    margin-left: auto;
  }
  .add-menu {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    padding: 0.25rem;
    min-width: 11rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    background-color: var(--color-surface-canvas);
  }
  .register-error {
    margin-bottom: 1rem;
    color: var(--color-feedback-error-text);
    font: var(--txt-body-s-regular);
  }
  /* A file list reads better as rows than as a stack of bordered cards: the
     release header is then the only card on the page. */
  .artifact {
    padding: 0.75rem 0.25rem;
    border-bottom: 1px solid var(--color-border-subtle);
  }
  .artifact:first-child {
    border-top: 1px solid var(--color-border-subtle);
  }
  .artifact-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }
  .summary {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
    background: none;
    border: 0;
    padding: 0;
    margin: 0;
    color: inherit;
    font: inherit;
    text-align: left;
    cursor: pointer;
  }
  .summary:hover .toggle,
  .summary:focus-visible .toggle {
    color: var(--color-text-primary);
  }
  /* One line that truncates rather than wraps: the chevron is the last item in
     the row, so it keeps sitting against the text, and a long title shortens
     the file name beside it instead of pushing the chevron onto its own line. */
  .identity {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
    flex-wrap: nowrap;
    min-width: 0;
  }
  /* The title is arbitrary-length prose, so it is the one that gives way. No
     floor on its width: one held the chevron away from a short title, and the
     60% cap on everything to its right already stops the title being squeezed
     out. */
  .name {
    font: var(--txt-body-l-regular);
    color: var(--color-text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
    flex-shrink: 1;
  }
  /* Held at its natural width: an identifier shortened to a character or two
     tells a reader nothing, so it keeps its own space and the title yields
     instead. The cap stops a long file name squeezing out the title. */
  /* Centred rather than baseline-aligned against the title: the chevron is an
     icon box with no text baseline of its own, so on a baseline it hangs by
     its bottom edge and rides visibly high above the title. */
  .trailing {
    display: inline-flex;
    align-items: baseline;
    align-self: center;
    gap: 0.5rem;
    flex-shrink: 0;
    max-width: 60%;
    min-width: 0;
  }
  .filename {
    font: var(--txt-body-m-regular);
    color: var(--color-text-tertiary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
  }
  .artifact-actions {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-left: auto;
    flex-shrink: 0;
  }
  .size {
    font: var(--txt-body-m-regular);
    color: var(--color-text-tertiary);
    white-space: nowrap;
    flex-shrink: 0;
  }
  /* Repeating the words "More info" down the whole list reads as noise, and
     hiding the control until hover leaves no sign it exists. A chevron that
     points down, then flips up, says accordion without either problem. */
  /* Opened, the row is no longer a one-line summary, so nothing is clipped:
     the title and file name wrap onto as many lines as they need. */
  .identity.expanded {
    flex-wrap: wrap;
  }
  .identity.expanded .trailing {
    max-width: none;
  }
  .identity.expanded .name,
  .identity.expanded .filename {
    white-space: normal;
    overflow: visible;
    text-overflow: clip;
    overflow-wrap: anywhere;
    max-width: none;
    min-width: 0;
  }
  .toggle {
    display: inline-flex;
    align-items: center;
    align-self: center;
    flex-shrink: 0;
    color: var(--color-text-tertiary);
    transition: transform 0.15s;
  }
  .toggle.open {
    transform: rotate(180deg);
  }
  .artifact-meta {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.75rem;
    margin-top: 0.375rem;
    font: var(--txt-body-s-regular);
  }
  /* A bare hash that copies on click, with no icon of its own: one per row
     down the list would be noise, and Id already explains itself on hover. */
  .cid {
    color: var(--color-text-tertiary);
    font: var(--txt-code-small);
  }
  .trust {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    color: var(--color-foreground-success);
    white-space: nowrap;
  }
  .seeding {
    display: inline-flex;
    align-items: center;
    height: 1.75rem;
    padding: 0 0.5rem;
    border: none;
    border-radius: var(--border-radius-sm);
    background: none;
    cursor: pointer;
    font: var(--txt-body-m-regular);
    color: var(--color-text-tertiary);
    white-space: nowrap;
  }
  .seeding:hover:not(:disabled),
  .seeding:focus-visible {
    background: var(--color-surface-subtle);
    color: var(--color-text-primary);
  }
  .seeding-label {
    display: inline-grid;
  }
  .seeding-idle,
  .seeding-stop {
    grid-area: 1 / 1;
    text-align: center;
  }
  .seeding-stop {
    visibility: hidden;
  }
  .seeding:hover:not(:disabled) .seeding-idle,
  .seeding:focus-visible .seeding-idle {
    visibility: hidden;
  }
  .seeding:hover:not(:disabled) .seeding-stop,
  .seeding:focus-visible .seeding-stop {
    visibility: visible;
  }
  .unseed-error {
    color: var(--color-foreground-red);
  }
  .contributor {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    color: var(--color-text-secondary);
  }
  /* A short fixed label, so it keeps its own width and never wraps: the title
     beside it is the part that gives way. */
  .redacted-badge {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    flex-shrink: 0;
    white-space: nowrap;
    font: var(--txt-body-s-regular);
    color: var(--color-text-tertiary);
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    padding: 0 0.375rem;
  }
  .details {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    margin-top: 0.75rem;
  }
  .section {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    font: var(--txt-body-m-regular);
  }
  /* Held at the height of the buttons some of these rows carry, so a heading
     does not jump when its button appears or goes away, and so every section
     heading sits at the same height whether or not it has one. */
  .section-title {
    display: flex;
    align-items: center;
    min-height: 1.5rem;
    gap: 0.5rem;
    font: var(--txt-body-s-medium);
    color: var(--color-text-secondary);
  }
  .section-count {
    color: var(--color-text-tertiary);
    font: var(--txt-body-s-regular);
  }
  .empty-section {
    color: var(--color-text-tertiary);
  }
  .people {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .person {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: var(--color-foreground-success);
  }
  .meta-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
    width: fit-content;
    max-width: 100%;
  }
  .meta-key {
    color: var(--color-text-tertiary);
    flex-shrink: 0;
  }
  .meta-value {
    word-break: break-word;
    min-width: 0;
  }
  .meta-actions {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    opacity: 0;
  }
  .meta-row:hover .meta-actions,
  .meta-row:focus-within .meta-actions {
    opacity: 1;
  }
  .meta-editor {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
  }
  /* Metadata keys and values are short; full-width inputs across the card
     overstate how much is meant to go in them. */
  .key-field {
    width: 9rem;
    flex-shrink: 0;
  }
  .value-field {
    width: 16rem;
    max-width: 100%;
  }
  .meta-error {
    color: var(--color-feedback-error-text);
    font: var(--txt-body-s-regular);
  }
  /* Matches the patch delete prompt, scaled down for a single entry. */
  .confirm-remove {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    padding: 0.75rem;
    min-width: 15rem;
    max-width: 22rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    background-color: var(--color-surface-canvas);
  }
  .confirm-remove-text {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    color: var(--color-text-primary);
  }
  .confirm-remove-note {
    color: var(--color-text-secondary);
  }
  .confirm-remove-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
  }
  .confirm-remove-button {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    height: 2rem;
    padding: 0 0.75rem;
    border: 0;
    border-radius: var(--border-radius-sm);
    background-color: var(--color-feedback-error-fill);
    color: var(--color-text-on-brand);
    cursor: pointer;
    transition: background-color 0.1s ease;
  }
  .confirm-remove-button:hover:not(:disabled),
  .confirm-remove-button:focus-visible:not(:disabled) {
    background-color: var(--color-feedback-error-fill-hover);
  }
  .confirm-remove-button:disabled {
    cursor: default;
    opacity: 0.6;
  }
  /* The enclosing .section already spaces a heading from its body, so these
     carry no margin of their own: with one they sat 8px lower than the empty
     states in the sections above them. */
  .locations,
  .redactions {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    font: var(--txt-body-m-regular);
  }
  .location-group {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    min-width: 0;
  }
  .location-node {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  .location-url {
    color: var(--color-text-secondary);
    word-break: break-all;
  }
  .url-field {
    width: 24rem;
    max-width: 100%;
  }
  /* The section heading already says these are redactions, so the row carries
     no warning icon and no alarm colour: it is a record of who did what. */
  .redaction {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.5rem;
    color: var(--color-text-secondary);
  }
  .reason {
    color: var(--color-text-tertiary);
    word-break: break-word;
  }
  .empty-artifacts {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.75rem;
    padding: 4rem 1.25rem;
    color: var(--color-text-tertiary);
    font: var(--txt-body-m-regular);
  }
</style>

<Layout>
  <div class="page">
    <Topbar>
      <div class="breadcrumb">
        <Icon name="parcel" />
        <button
          class="breadcrumb-link"
          onclick={() =>
            router.push({
              resource: "repo.releases",
              rid: repo.rid,
            })}>
          Releases
        </button>
        <Icon name="chevron-right" />
        {#if scope}
          <button
            class="breadcrumb-link"
            onclick={() =>
              router.push({
                resource: "repo.releases",
                rid: repo.rid,
                scope,
              })}>
            {scope === "trusted" ? "Delegates" : "Others"}
          </button>
          <Icon name="chevron-right" />
        {/if}
        <Id id={release.id} clipboard={release.id} label="release ID" />
      </div>
      <div style:margin-left="auto" style:display="flex" style:gap="0.5rem">
        {#if release.creator.did === ownDid}
          <Button styleHeight="2rem" variant="naked" onclick={openDelete}>
            <Icon name="trash" />Delete
          </Button>
        {/if}
        <ShareButton
          target={{
            type: "cob",
            kind: "release",
            rid: repo.rid,
            oid: release.id,
          }}
          id={release.id}
          idLabel="release"
          variant="naked"
          {config} />
        <Popover placement="bottom-end" popoverPadding="0">
          {#snippet toggle(onclick)}
            <Button
              styleHeight="2rem"
              variant="naked"
              disabled={registering}
              {onclick}>
              <Icon name="plus" />
              {registering ? "Registering…" : "Register artifacts"}
            </Button>
          {/snippet}
          {#snippet popover()}
            <div class="add-menu">
              <Button
                variant="naked"
                styleWidth="100%"
                styleJustifyContent="flex-start"
                onclick={chooseFiles}>
                <Icon name="attach" />Files…
              </Button>
              <Button
                variant="naked"
                styleWidth="100%"
                styleJustifyContent="flex-start"
                onclick={chooseFolder}>
                <Icon name="folder" />Folder…
              </Button>
            </div>
          {/snippet}
        </Popover>
      </div>
    </Topbar>

    <ScrollArea style="flex: 1; min-height: 0;">
      <div class="main">
        <div class="title">
          <div class="global-chip title-chip">
            <Icon name="parcel" />
          </div>
          <InlineTitle content={title} fontSize="large" />
        </div>

        <div class="metadata-row">
          <ReleaseMetadata {release} {repo} {delegateIds} />
        </div>

        {#if view.showFilters || view.redactedCount > 0}
          <div class="filter">
            {#if view.showFilters}
              <Button
                styleHeight="1.75rem"
                bordered
                flatRight
                active={view.scope === "trusted"}
                onclick={() =>
                  router.push({
                    resource: "repo.release",
                    rid: repo.rid,
                    release: release.id,
                    scope,
                  })}>
                <Icon name="badge" />Delegates
                <span class="global-counter-badge">{view.counts.trusted}</span>
              </Button>
              <Button
                styleHeight="1.75rem"
                bordered
                flatLeft
                active={view.scope === "untrusted"}
                title="Non-delegates"
                onclick={() =>
                  router.push({
                    resource: "repo.release",
                    rid: repo.rid,
                    release: release.id,
                    scope,
                    artifactScope: "untrusted",
                  })}>
                <Icon name="avatar-incognito" />Others
                <span class="global-counter-badge">
                  {view.counts.untrusted}
                </span>
              </Button>
            {/if}
            {#if view.redactedCount > 0}
              <div class="redacted-toggle">
                <Button
                  styleHeight="1.75rem"
                  variant="naked"
                  onclick={() => (showRedacted = !showRedacted)}>
                  {showRedacted ? "Hide redacted" : "Show redacted"}
                  <span class="global-counter-badge">{view.redactedCount}</span>
                </Button>
              </div>
            {/if}
          </div>
        {/if}

        {#if registerError}
          <div class="register-error">{registerError}</div>
        {/if}

        {#if view.shown.length === 0}
          <div class="empty-artifacts">
            <Icon name="attach" />
            No artifacts
          </div>
        {/if}

        <div class="artifact-list">
          {#each view.shown as artifact (artifact.cid)}
            {@const size = artifactSize(artifact)}
            {@const locations = delegatesFirst(
              locationsByNode(artifact.locations),
              g => g.user.did,
              delegateIds,
            )}
            {@const metadata = otherMetadata(artifact)}
            {@const locationCount = artifact.locations.length}
            {@const redactions = delegatesFirst(
              artifact.redactions,
              r => r.user.did,
              delegateIds,
            )}
            {@const attestations = delegatesFirst(
              artifact.attestations,
              n => n.did,
              delegateIds,
            )}
            {@const isOpen = expanded[artifact.cid] === true}
            {@const editable = canEditMetadata(artifact, ownDid, delegateIds)}
            {@const artifactName = artifactTitle(artifact)}
            {@const trust = attestationLabel(attestations)}
            <div class="artifact">
              <div class="artifact-row">
                <!-- The whole title is the disclosure, with the chevron as its
                     indicator, so the hit target matches what a reader would
                     aim at. -->
                <button
                  type="button"
                  class="summary"
                  aria-expanded={isOpen}
                  title={isOpen ? "Hide details" : "Show details"}
                  onclick={() => toggleExpanded(artifact.cid)}>
                  <span class="identity" class:expanded={isOpen}>
                    <span class="name">{artifactName ?? artifact.name}</span>
                    <!-- One unit, so a line break can never leave the chevron
                         stranded on a line of its own. -->
                    <span class="trailing">
                      {#if artifactName}
                        <span class="filename">{artifact.name}</span>
                      {/if}
                      {#if artifact.redacted}
                        <span class="redacted-badge">
                          {redactedByDelegate(artifact, delegateIds)
                            ? "Redacted by delegate"
                            : "Redacted by author"}
                        </span>
                      {/if}
                      <span class="toggle" class:open={isOpen}>
                        <Icon name="chevron-down" />
                      </span>
                    </span>
                  </span>
                </button>
                {#if size}
                  <span class="size">{size}</span>
                {/if}
                <div class="artifact-actions">
                  {#if seededCids.has(artifact.cid)}
                    <button
                      class="seeding"
                      disabled={unseeding.has(artifact.cid)}
                      title="Your node is seeding this artifact. Click to unseed."
                      onclick={() => stopSeeding(artifact.cid)}>
                      {#if unseeding.has(artifact.cid)}
                        Unseeding…
                      {:else}
                        <span class="seeding-label">
                          <span class="seeding-idle">Seeding</span>
                          <span class="seeding-stop">Unseed</span>
                        </span>
                      {/if}
                    </button>
                  {:else if !artifact.redacted}
                    <Button
                      variant="naked"
                      styleHeight="1.75rem"
                      disabled={reseeding === artifact.cid ||
                        artifactNodeUp === false}
                      title={artifactNodeUp === false
                        ? "Your artifact node is not running"
                        : "Seed your own copy so others can download it"}
                      onclick={() => reseed(artifact)}>
                      {reseeding === artifact.cid ? "Seeding…" : "Seed"}
                    </Button>
                  {/if}
                  <ArtifactDownloadButton
                    {artifact}
                    {delegateIds}
                    seeding={seededCids.has(artifact.cid)}
                    onDownloaded={refreshSeeded}
                    releaseId={release.id}
                    rid={repo.rid} />
                </div>
              </div>

              <div class="artifact-meta">
                <span class="cid">
                  <Id
                    id={cidLabels.get(artifact.cid) ?? artifact.cid}
                    clipboard={artifact.cid}
                    label="CID"
                    shorten={false} />
                </span>
                <span class="contributor">
                  <NodeId {...authorForNodeId(artifact.author)} />
                  {#if delegateIds.has(artifact.author.did)}
                    <DelegateBadge />
                  {:else if artifact.author.did === ownDid}
                    <YouBadge />
                  {/if}
                </span>
                {#if trust}
                  <span class="trust">
                    <Icon name="checkmark" />
                    {trust}
                  </span>
                {/if}
                {#if reseedError?.cid === artifact.cid}
                  <span class="unseed-error">{reseedError.message}</span>
                {/if}
                {#if unseedFailed === artifact.cid}
                  <span class="unseed-error">Could not unseed.</span>
                {/if}
              </div>

              {#if isOpen}
                <div class="details" transition:slide={{ duration: 180 }}>
                  <div class="section">
                    <div class="section-title">
                      Metadata
                      <span class="section-count">{metadata.length}</span>
                      {#if editable && editing !== editorId(artifact.cid, "")}
                        <Button
                          variant="naked"
                          styleHeight="1.5rem"
                          disabled={saving}
                          onclick={() =>
                            startEdit(artifact.cid, "", undefined)}>
                          <Icon name="plus" />Add
                        </Button>
                      {/if}
                    </div>

                    {#if metadata.length === 0 && editing !== editorId(artifact.cid, "")}
                      <div class="empty-section">No metadata</div>
                    {/if}

                    {#each metadata as [key, value] (key)}
                      {#if editing === editorId(artifact.cid, key)}
                        <div class="meta-editor">
                          <span class="key-field">
                            <TextInput
                              bind:value={draftKey}
                              placeholder="Key"
                              styleHeight="1.75rem"
                              disabled={saving} />
                          </span>
                          <span class="value-field">
                            <TextInput
                              bind:value={draftValue}
                              placeholder="Value"
                              styleHeight="1.75rem"
                              disabled={saving}
                              onSubmit={() => saveMetadata(artifact, key)} />
                          </span>
                          <Button
                            variant="secondary"
                            styleHeight="1.75rem"
                            disabled={saving}
                            onclick={() => saveMetadata(artifact, key)}>
                            Save
                          </Button>
                          <Button
                            variant="naked"
                            styleHeight="1.75rem"
                            disabled={saving}
                            onclick={cancelEdit}>
                            Cancel
                          </Button>
                        </div>
                      {:else}
                        <div class="meta-row">
                          <span class="meta-key">{key}</span>
                          <span class="meta-value">
                            {displayMetadataValue(value)}
                          </span>
                          {#if editable}
                            <span class="meta-actions">
                              <Button
                                variant="naked"
                                styleHeight="1.5rem"
                                title="Edit"
                                disabled={saving}
                                onclick={() =>
                                  startEdit(artifact.cid, key, value)}>
                                <Icon name="edit" />
                              </Button>
                              <Popover
                                placement="bottom-end"
                                popoverPadding="0">
                                {#snippet toggle(onclick)}
                                  <Button
                                    variant="naked"
                                    styleHeight="1.5rem"
                                    title="Remove"
                                    disabled={saving}
                                    {onclick}>
                                    <Icon name="trash" />
                                  </Button>
                                {/snippet}
                                {#snippet popover()}
                                  <div class="confirm-remove">
                                    <div class="confirm-remove-text">
                                      <div class="txt-body-m-medium">
                                        Remove "{key}"?
                                      </div>
                                      <div
                                        class="confirm-remove-note txt-body-m-regular">
                                        The entry is dropped from the release
                                        for everyone who replicates it. You can
                                        set it again afterwards.
                                      </div>
                                    </div>
                                    <div class="confirm-remove-actions">
                                      <Button
                                        variant="outline"
                                        disabled={saving}
                                        onclick={closeFocused}>
                                        Cancel
                                      </Button>
                                      <button
                                        type="button"
                                        class="confirm-remove-button txt-body-m-medium"
                                        disabled={saving}
                                        onclick={() =>
                                          removeMetadata(artifact, key)}>
                                        <Icon name="trash" />
                                        {saving ? "Removing…" : "Remove"}
                                      </button>
                                    </div>
                                  </div>
                                {/snippet}
                              </Popover>
                            </span>
                          {/if}
                        </div>
                      {/if}
                    {/each}

                    {#if editing === editorId(artifact.cid, "")}
                      <div class="meta-editor">
                        <span class="key-field">
                          <TextInput
                            bind:value={draftKey}
                            placeholder="Key"
                            autofocus
                            styleHeight="1.75rem"
                            disabled={saving} />
                        </span>
                        <span class="value-field">
                          <TextInput
                            bind:value={draftValue}
                            placeholder="Value"
                            styleHeight="1.75rem"
                            disabled={saving}
                            onSubmit={() => saveMetadata(artifact, "")} />
                        </span>
                        <Button
                          variant="secondary"
                          styleHeight="1.75rem"
                          disabled={saving}
                          onclick={() => saveMetadata(artifact, "")}>
                          Save
                        </Button>
                        <Button
                          variant="naked"
                          styleHeight="1.75rem"
                          disabled={saving}
                          onclick={cancelEdit}>
                          Cancel
                        </Button>
                      </div>
                    {/if}

                    {#if metadataError}
                      <div class="meta-error">{metadataError}</div>
                    {/if}
                  </div>

                  <div class="section">
                    <div class="section-title">
                      Attestations
                      <span class="section-count">{attestations.length}</span>
                      {#if canAttest(artifact, ownDid)}
                        <Button
                          variant="naked"
                          styleHeight="1.5rem"
                          disabled={attesting === artifact.cid}
                          title="Attest that your own build has the same CID"
                          onclick={() => attest(artifact)}>
                          <Icon name="checkmark" />
                          {attesting === artifact.cid ? "Attesting…" : "Attest"}
                        </Button>
                      {/if}
                    </div>
                    {#if attestError?.cid === artifact.cid}
                      <div class="meta-error">{attestError.message}</div>
                    {/if}
                    {#if attestations.length === 0}
                      <div class="empty-section">
                        Nobody has attested to this artifact
                      </div>
                    {:else}
                      <div class="people">
                        {#each attestations as node (node.did)}
                          <div class="person">
                            <Icon name="checkmark" />
                            <NodeId {...authorForNodeId(node)} />
                            {#if delegateIds.has(node.did)}
                              <DelegateBadge tooltip="Attested by a delegate" />
                            {/if}
                          </div>
                        {/each}
                      </div>
                    {/if}
                  </div>

                  <div class="section">
                    <div class="section-title">
                      Redactions
                      <span class="section-count">{redactions.length}</span>
                      <!-- Offered only to the artifact's author and delegates.
                           Anyone may redact, but only those two hide it from
                           the release; for everyone else it would burn their
                           own ability to attest and change nothing on screen.
                           Hidden once this node has redacted, since a second
                           one only rewrites the reason. -->
                      {#if canEditMetadata(artifact, ownDid, delegateIds) && !redactions.some(r => r.user.did === ownDid)}
                        <Button
                          variant="naked"
                          styleHeight="1.5rem"
                          onclick={() => openRedact(artifact)}>
                          <Icon name="warning" />Redact
                        </Button>
                      {/if}
                    </div>
                    {#if redactions.length === 0}
                      <div class="empty-section">No redactions</div>
                    {:else}
                      <div class="redactions">
                        {#each redactions as redaction (redaction.user.did)}
                          <div class="redaction">
                            <NodeId {...authorForNodeId(redaction.user)} />
                            {#if delegateIds.has(redaction.user.did)}
                              <DelegateBadge tooltip="Redacted by a delegate" />
                            {/if}
                            <span class="reason">
                              {redaction.reason || "No reason"}
                            </span>
                          </div>
                        {/each}
                      </div>
                    {/if}
                  </div>

                  <div class="section">
                    <div class="section-title">
                      Locations
                      <span class="section-count">{locationCount}</span>
                      {#if !artifact.redacted && addingLocation !== artifact.cid}
                        <Button
                          variant="naked"
                          styleHeight="1.5rem"
                          disabled={savingLocation}
                          title="Add a URL others can download this artifact from"
                          onclick={() => startAddLocation(artifact.cid)}>
                          <Icon name="plus" />Add
                        </Button>
                      {/if}
                    </div>
                    {#if addingLocation === artifact.cid}
                      <div class="meta-editor">
                        <span class="url-field">
                          <TextInput
                            bind:value={draftUrl}
                            placeholder="https://…"
                            autofocus
                            styleHeight="1.75rem"
                            disabled={savingLocation}
                            onDismiss={cancelAddLocation}
                            onSubmit={() => addLocation(artifact.cid)} />
                        </span>
                        <Button
                          variant="secondary"
                          styleHeight="1.75rem"
                          disabled={savingLocation}
                          onclick={() => addLocation(artifact.cid)}>
                          Save
                        </Button>
                        <Button
                          variant="naked"
                          styleHeight="1.75rem"
                          disabled={savingLocation}
                          onclick={cancelAddLocation}>
                          Cancel
                        </Button>
                      </div>
                    {/if}
                    {#if locationError?.cid === artifact.cid}
                      <div class="meta-error">{locationError.message}</div>
                    {/if}
                    {#if locationCount === 0}
                      {#if addingLocation !== artifact.cid}
                        <div class="empty-section">No locations</div>
                      {/if}
                    {:else}
                      <div class="locations">
                        {#each locations as group (group.user.did)}
                          <div class="location-group">
                            <div class="location-node">
                              <NodeId {...authorForNodeId(group.user)} />
                              {#if delegateIds.has(group.user.did)}
                                <DelegateBadge
                                  tooltip="Location added by a delegate" />
                              {/if}
                            </div>
                            {#each group.urls as url (url)}
                              <div class="meta-row">
                                <span class="location-url">
                                  <Id
                                    id={url}
                                    clipboard={url}
                                    label="location URL"
                                    shorten={false} />
                                </span>
                                {#if group.user.did === ownDid}
                                  <span class="meta-actions">
                                    <Popover
                                      placement="bottom-end"
                                      popoverPadding="0">
                                      {#snippet toggle(onclick)}
                                        <Button
                                          variant="naked"
                                          styleHeight="1.5rem"
                                          title="Remove location"
                                          disabled={savingLocation}
                                          {onclick}>
                                          <Icon name="trash" />
                                        </Button>
                                      {/snippet}
                                      {#snippet popover()}
                                        <!-- Withdrawing the node's own endpoint
                                             while it still seeds would leave a
                                             tag nobody can find, so the two go
                                             together, as in Unseed. -->
                                        {@const seeding =
                                          url.startsWith("radiroh:") &&
                                          seededCids.has(artifact.cid)}
                                        {@const busy =
                                          savingLocation ||
                                          unseeding.has(artifact.cid)}
                                        <div class="confirm-remove">
                                          <div class="confirm-remove-text">
                                            <div class="txt-body-m-medium">
                                              {seeding
                                                ? "Unseed this artifact?"
                                                : "Remove this location?"}
                                            </div>
                                            <div
                                              class="confirm-remove-note txt-body-m-regular">
                                              {#if seeding}
                                                This is your node's location.
                                                Removing it also unseeds it from
                                                your node seeding the artifact.
                                              {:else}
                                                Peers stop downloading the
                                                artifact from it.
                                              {/if}
                                            </div>
                                          </div>
                                          <div class="confirm-remove-actions">
                                            <Button
                                              variant="outline"
                                              disabled={busy}
                                              onclick={closeFocused}>
                                              Cancel
                                            </Button>
                                            <button
                                              type="button"
                                              class="confirm-remove-button txt-body-m-medium"
                                              disabled={busy}
                                              onclick={async () => {
                                                if (seeding) {
                                                  await stopSeeding(
                                                    artifact.cid,
                                                  );
                                                  closeFocused();
                                                } else {
                                                  await removeLocation(
                                                    artifact.cid,
                                                    url,
                                                  );
                                                }
                                              }}>
                                              <Icon name="trash" />
                                              {#if seeding}
                                                {busy ? "Unseeding…" : "Unseed"}
                                              {:else}
                                                {busy ? "Removing…" : "Remove"}
                                              {/if}
                                            </button>
                                          </div>
                                        </div>
                                      {/snippet}
                                    </Popover>
                                  </span>
                                {/if}
                              </div>
                            {/each}
                          </div>
                        {/each}
                      </div>
                    {/if}
                  </div>
                </div>
              {/if}
            </div>
          {/each}
        </div>
      </div>
    </ScrollArea>
  </div>
</Layout>
