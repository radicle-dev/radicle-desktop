<script lang="ts">
  import type { Author } from "@bindings/cob/Author";
  import type { Artifact } from "@bindings/cob/release/Artifact";
  import type { Release } from "@bindings/cob/release/Release";
  import type { Config } from "@bindings/config/Config";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import { slide } from "svelte/transition";

  import { invoke } from "@app/lib/invoke";
  import * as router from "@app/lib/router";
  import {
    artifactPlatform,
    authorForNodeId,
    didFromPublicKey,
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

  import Layout from "./Layout.svelte";

  interface Props {
    repo: RepoInfo;
    config: Config;
    release: Release;
    allAuthors: boolean;
  }

  /* eslint-disable prefer-const */
  let { repo, config, release, allAuthors }: Props = $props();
  /* eslint-enable prefer-const */

  const SIZE_KEY = "sizeBytes";

  const ownDid = $derived(didFromPublicKey(config.publicKey));

  // A release COB carries no name of its own. The backend resolves one from the
  // annotated tag's message, falling back to the commit subject; failing both,
  // the tag name and then the release id stand in.
  const title = $derived(release.title || release.tagName || release.id);
  const delegateIds = $derived(new Set(repo.delegates.map(d => d.did)));

  // An artifact redacted by its own author or a delegate is hidden by default.
  let showRedacted = $state(false);

  // Whether the artifact was redacted by a trusted party (its author or a
  // delegate), mirroring the backend's default-hidden rule.
  function redactedByTrusted(artifact: Artifact, delegates: Set<string>) {
    return artifact.redactions.some(
      r => r.user.did === artifact.author.did || delegates.has(r.user.did),
    );
  }

  // Label for the redacted badge. Delegate takes precedence over author,
  // since the author may also be a delegate.
  function redactedByLabel(artifact: Artifact, delegates: Set<string>): string {
    const byDelegate = artifact.redactions.some(r => delegates.has(r.user.did));
    return byDelegate ? "Redacted by delegate" : "Redacted by author";
  }

  // Artifacts visible under the given redaction toggle.
  function visible(
    list: Artifact[],
    show: boolean,
    delegates: Set<string>,
  ): Artifact[] {
    return show ? list : list.filter(a => !redactedByTrusted(a, delegates));
  }

  const delegateArtifacts = $derived(
    release.artifacts.filter(a => delegateIds.has(a.author.did)),
  );
  // Artifacts are scoped to delegate authors by default; the command returns
  // all of them, so filtering happens here. With no delegate artifact at all,
  // fall back to every author.
  const authorArtifacts = $derived(
    allAuthors || delegateArtifacts.length === 0
      ? release.artifacts
      : delegateArtifacts,
  );
  const shownArtifacts = $derived(
    visible(authorArtifacts, showRedacted, delegateIds),
  );
  // The redacted count is the hidden set within the current author scope.
  const redactedCount = $derived(
    authorArtifacts.filter(a => redactedByTrusted(a, delegateIds)).length,
  );

  // Segment counts reflect what each choice would actually show, so an
  // artifact hidden as redacted never counts towards a scope.
  const delegateCount = $derived(
    visible(delegateArtifacts, showRedacted, delegateIds).length,
  );
  const allCount = $derived(
    visible(release.artifacts, showRedacted, delegateIds).length,
  );

  // Filter only when both scopes hold something.
  const showFilters = $derived(
    delegateArtifacts.length > 0 &&
      delegateArtifacts.length !== release.artifacts.length,
  );

  // Format a byte count as a human-readable size (mirrors the CLI display).
  function formatBytes(bytes: number): string {
    const units = ["B", "KiB", "MiB", "GiB", "TiB"];
    let value = bytes;
    let i = 0;
    while (value >= 1024 && i < units.length - 1) {
      value /= 1024;
      i += 1;
    }
    return `${i === 0 ? value : value.toFixed(1)} ${units[i]}`;
  }

  function artifactSize(artifact: Artifact): string | undefined {
    const size = artifact.metadata[SIZE_KEY];
    return typeof size === "number" ? formatBytes(size) : undefined;
  }

  // Group an artifact's locations by the contributing node, preserving order.
  function locationsByNode(artifact: Artifact) {
    const order: string[] = [];
    const groups: Record<
      string,
      { user: Artifact["locations"][number]["user"]; urls: string[] }
    > = {};
    for (const { user, url } of artifact.locations) {
      if (!groups[user.did]) {
        groups[user.did] = { user, urls: [] };
        order.push(user.did);
      }
      groups[user.did].urls.push(url);
    }
    return order.map(did => groups[did]);
  }

  // Order a node-keyed list so delegates come first. The backend returns these
  // sorted by DID (arbitrary to a reader); sort is stable, so the original
  // order is preserved within each group.
  function delegatesFirst<T>(
    items: T[],
    did: (item: T) => string,
    delegates: Set<string>,
  ): T[] {
    return [...items].sort(
      (a, b) => Number(delegates.has(did(b))) - Number(delegates.has(did(a))),
    );
  }

  // Metadata entries other than the size hint, shown as raw key/value pairs.
  function otherMetadata(artifact: Artifact): [string, unknown][] {
    return Object.entries(artifact.metadata).filter(
      ([key]) => key !== SIZE_KEY,
    );
  }

  // The trust signal worth carrying on the collapsed row. Delegate attestations
  // are what the repo's own maintainers vouched for, so they are named apart
  // from attestations by anyone else.
  function attestationLabel(
    attestations: Author[],
    delegates: Set<string>,
  ): string | undefined {
    if (attestations.length === 0) {
      return undefined;
    }
    const byDelegates = attestations.filter(a => delegates.has(a.did)).length;
    if (byDelegates > 0) {
      return `Verified by ${byDelegates} delegate${byDelegates === 1 ? "" : "s"}`;
    }
    return `Attested by ${attestations.length} node${
      attestations.length === 1 ? "" : "s"
    }`;
  }

  // Computed across the artifacts actually on screen, so the shortest form
  // that tells them apart is the one shown.
  const cidLabels = $derived(shortenCids(shownArtifacts.map(a => a.cid)));

  // Which artifact cards are expanded, keyed by content id.
  const expanded: Record<string, boolean> = $state({});

  function toggleExpanded(cid: string) {
    expanded[cid] = !expanded[cid];
  }

  // Writing metadata is constrained to the artifact's author or a repository
  // delegate, so the editor stays hidden for everyone else.
  function canEditMetadata(artifact: Artifact): boolean {
    return artifact.author.did === ownDid || delegateIds.has(ownDid);
  }

  // The COB stores values as free-form JSON. Text that parses as JSON is sent
  // as that value, so numbers and booleans round-trip; anything else is sent
  // as a plain string.
  function parseValue(input: string): unknown {
    try {
      return JSON.parse(input);
    } catch {
      return input;
    }
  }

  function displayValue(value: unknown): string {
    return typeof value === "string" ? value : JSON.stringify(value);
  }

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
    draftValue = key === "" ? "" : displayValue(value);
    metadataError = undefined;
  }

  function cancelEdit() {
    editing = undefined;
    draftKey = "";
    draftValue = "";
    metadataError = undefined;
  }

  async function reload() {
    const updated = await invoke<Release | null>("release_by_id", {
      rid: repo.rid,
      id: release.id,
    });
    if (updated) {
      release = updated;
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
        value: parseValue(draftValue),
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
  /* A file list reads better as rows than as a stack of bordered cards: the
     release header is then the only card on the page. */
  .artifact-list {
    border-top: 1px solid var(--color-border-subtle);
  }
  .artifact {
    padding: 0.75rem 0.25rem;
    border-bottom: 1px solid var(--color-border-subtle);
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
  .identity {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
    flex-wrap: wrap;
    min-width: 0;
  }
  .platform {
    font: var(--txt-body-l-regular);
    color: var(--color-text-primary);
    word-break: break-word;
  }
  .filename {
    font: var(--txt-body-m-regular);
    color: var(--color-text-tertiary);
    word-break: break-all;
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
  }
  /* Repeating the words "More info" down the whole list reads as noise, and
     hiding the control until hover leaves no sign it exists. A chevron that
     points down, then flips up, says accordion without either problem. */
  .toggle {
    display: inline-flex;
    align-items: center;
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
  .contributor {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    color: var(--color-text-secondary);
  }
  .redacted-badge {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    font: var(--txt-body-s-regular);
    color: var(--color-feedback-error-text);
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
  .section-title {
    display: flex;
    align-items: center;
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
  .locations,
  .redactions {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    margin-top: 0.5rem;
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
  .redaction {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.5rem;
    color: var(--color-feedback-error-text);
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
              allAuthors,
            })}>
          Releases
        </button>
        <Icon name="chevron-right" />
        <Id id={release.id} clipboard={release.id} label="release ID" />
      </div>
      <div style:margin-left="auto" style:display="flex" style:gap="0.5rem">
        <ShareButton
          explorerPath={`${repo.rid}/releases/${release.id}`}
          id={release.id}
          idLabel="release"
          variant="naked"
          {config} />
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

        {#if showFilters || redactedCount > 0}
          <div class="filter">
            {#if showFilters}
              <Button
                styleHeight="1.75rem"
                bordered
                flatRight
                active={!allAuthors}
                onclick={() =>
                  router.push({
                    resource: "repo.release",
                    rid: repo.rid,
                    release: release.id,
                    allAuthors: false,
                  })}>
                <Icon name="badge" />Delegates
                <span class="global-counter-badge">{delegateCount}</span>
              </Button>
              <Button
                styleHeight="1.75rem"
                bordered
                flatLeft
                active={allAuthors}
                onclick={() =>
                  router.push({
                    resource: "repo.release",
                    rid: repo.rid,
                    release: release.id,
                    allAuthors: true,
                  })}>
                <Icon name="avatar-incognito" />All
                <span class="global-counter-badge">{allCount}</span>
              </Button>
            {/if}
            {#if redactedCount > 0}
              <div class="redacted-toggle">
                <Button
                  styleHeight="1.75rem"
                  variant="naked"
                  onclick={() => (showRedacted = !showRedacted)}>
                  {showRedacted ? "Hide redacted" : "Show redacted"}
                  <span class="global-counter-badge">{redactedCount}</span>
                </Button>
              </div>
            {/if}
          </div>
        {/if}

        {#if shownArtifacts.length === 0}
          <div class="empty-artifacts">
            <Icon name="attach" />
            No artifacts
          </div>
        {/if}

        <div class="artifact-list">
          {#each shownArtifacts as artifact (artifact.cid)}
            {@const size = artifactSize(artifact)}
            {@const locations = delegatesFirst(
              locationsByNode(artifact),
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
            {@const editable = canEditMetadata(artifact)}
            {@const platform = artifactPlatform(artifact.name)}
            {@const trust = attestationLabel(attestations, delegateIds)}
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
                  <span class="identity">
                    {#if platform}
                      <span class="platform">{platform}</span>
                      <span class="filename">{artifact.name}</span>
                    {:else}
                      <span class="platform">{artifact.name}</span>
                    {/if}
                    {#if redactedByTrusted(artifact, delegateIds)}
                      <span class="redacted-badge">
                        <Icon name="warning" />
                        {redactedByLabel(artifact, delegateIds)}
                      </span>
                    {/if}
                  </span>
                  <span class="toggle" class:open={isOpen}>
                    <Icon name="chevron-down" />
                  </span>
                </button>
                <div class="artifact-actions">
                  {#if size}
                    <span class="size">{size}</span>
                  {/if}
                  <ArtifactDownloadButton
                    {artifact}
                    {delegateIds}
                    releaseId={release.id}
                    rid={repo.rid} />
                </div>
              </div>

              <div class="artifact-meta">
                <!-- Shown in full: unlike a git oid, the leading characters of a
                   content id are a multihash prefix shared by every artifact,
                   so a truncated one identifies nothing. -->
                <span class="cid">
                  <Id
                    id={cidLabels.get(artifact.cid) ?? artifact.cid}
                    clipboard={artifact.cid}
                    label="content ID"
                    shorten={false} />
                </span>
                {#if trust}
                  <span class="trust">
                    <Icon name="checkmark" />
                    {trust}
                  </span>
                {/if}
                <!-- The release creator is named in the header, so an artifact
                   only names its own author when somebody else contributed it. -->
                {#if artifact.author.did !== release.creator.did}
                  <span class="contributor">
                    <NodeId {...authorForNodeId(artifact.author)} />
                    {#if delegateIds.has(artifact.author.did)}
                      <DelegateBadge />
                    {/if}
                  </span>
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
                          <span class="meta-value">{displayValue(value)}</span>
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
                    </div>
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
                    </div>
                    {#if redactions.length === 0}
                      <div class="empty-section">No redactions</div>
                    {:else}
                      <div class="redactions">
                        {#each redactions as redaction (redaction.user.did)}
                          <div class="redaction">
                            <Icon name="warning" />
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
                    </div>
                    {#if locationCount === 0}
                      <div class="empty-section">
                        No location has been announced
                      </div>
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
                              <div class="location-url">{url}</div>
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
