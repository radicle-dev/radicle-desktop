<script lang="ts">
  import type { Artifact } from "@bindings/cob/release/Artifact";
  import type { Release } from "@bindings/cob/release/Release";
  import type { Config } from "@bindings/config/Config";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import * as router from "@app/lib/router";
  import { authorForNodeId } from "@app/lib/utils";

  import ArtifactDownloadButton from "@app/components/ArtifactDownloadButton.svelte";
  import Button from "@app/components/Button.svelte";
  import DelegateBadge from "@app/components/DelegateBadge.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import InlineTitle from "@app/components/InlineTitle.svelte";
  import NodeId from "@app/components/NodeId.svelte";
  import ReleaseMetadata from "@app/components/ReleaseMetadata.svelte";
  import ScrollArea from "@app/components/ScrollArea.svelte";
  import ShareButton from "@app/components/ShareButton.svelte";
  import Topbar from "@app/components/Topbar.svelte";

  import Layout from "./Layout.svelte";

  interface Props {
    repo: RepoInfo;
    config: Config;
    release: Release;
    allAuthors: boolean;
  }

  const { repo, config, release, allAuthors }: Props = $props();

  const SIZE_KEY = "sizeBytes";

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
  .breadcrumb-title {
    color: var(--color-text-primary);
    font: var(--txt-body-m-medium);
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
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
  .artifact {
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    background-color: var(--color-surface-base);
    padding: 1rem;
    margin-bottom: 0.75rem;
  }
  .artifact-name {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
    font: var(--txt-body-l-regular);
    word-break: break-word;
  }
  .artifact-actions {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-left: auto;
  }
  .size {
    font: var(--txt-body-m-regular);
    color: var(--color-text-tertiary);
    white-space: nowrap;
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
  .provenance {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.5rem;
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
    margin-top: 0.5rem;
  }
  .attestor {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
  }
  .metadata {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    margin-top: 0.75rem;
    font: var(--txt-body-m-regular);
  }
  .meta-key {
    color: var(--color-text-tertiary);
  }
  details {
    margin-top: 0.75rem;
  }
  summary {
    cursor: pointer;
    font: var(--txt-body-m-regular);
    color: var(--color-text-tertiary);
  }
  summary:hover {
    color: var(--color-text-primary);
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
        <span class="breadcrumb-title">{title}</span>
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
          <div class="artifact">
            <div class="artifact-name">
              <span>{artifact.name}</span>
              {#if redactedByTrusted(artifact, delegateIds)}
                <span class="redacted-badge">
                  <Icon name="warning" />
                  {redactedByLabel(artifact, delegateIds)}
                </span>
              {/if}
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

            <div style:margin-top="0.25rem">
              <Id
                id={artifact.cid}
                clipboard={artifact.cid}
                label="content ID" />
            </div>

            <div class="provenance">
              <NodeId {...authorForNodeId(artifact.author)} />
              {#if delegateIds.has(artifact.author.did)}
                <DelegateBadge />
              {/if}
              {#if artifact.attestations.length > 0}
                attested by
                {#each delegatesFirst(artifact.attestations, n => n.did, delegateIds) as node, i (node.did)}
                  <span class="attestor">
                    <NodeId {...authorForNodeId(node)} />
                    {#if delegateIds.has(node.did)}
                      <DelegateBadge />
                    {/if}{#if i < artifact.attestations.length - 1},{/if}
                  </span>
                {/each}
              {/if}
            </div>

            {#if metadata.length > 0}
              <div class="metadata">
                {#each metadata as [key, value] (key)}
                  <span>
                    <span class="meta-key">{key}</span>
                    {typeof value === "string" ? value : JSON.stringify(value)}
                  </span>
                {/each}
              </div>
            {/if}

            {#if locationCount > 0}
              <details>
                <summary>
                  {locationCount} location{locationCount === 1 ? "" : "s"}
                </summary>
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
              </details>
            {/if}

            {#if redactions.length > 0}
              <details>
                <summary>
                  {redactions.length} redaction{redactions.length === 1
                    ? ""
                    : "s"}
                </summary>
                <div class="redactions">
                  {#each redactions as redaction (redaction.user.did)}
                    <div class="redaction">
                      <Icon name="warning" />
                      Redacted by
                      <NodeId {...authorForNodeId(redaction.user)} />
                      {#if delegateIds.has(redaction.user.did)}
                        <DelegateBadge />
                      {/if}
                      <span class="reason">
                        {redaction.reason || "No reason"}
                      </span>
                    </div>
                  {/each}
                </div>
              </details>
            {/if}
          </div>
        {/each}
      </div>
    </ScrollArea>
  </div>
</Layout>
