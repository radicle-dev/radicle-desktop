<script lang="ts">
  import type { Identity } from "@bindings/identity/Identity";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import { slide } from "svelte/transition";

  import { show } from "@app/lib/modal";
  import {
    absoluteTimestamp,
    authorForNodeId,
    formatTimestamp,
    pluralize,
  } from "@app/lib/utils";

  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import IdentityChanges from "@app/components/IdentityChanges.svelte";
  import IdentityDocument from "@app/components/IdentityDocument.svelte";
  import IdentityStateBadge, {
    stateCaption,
    stateIcon,
  } from "@app/components/IdentityStateBadge.svelte";
  import Markdown from "@app/components/Markdown.svelte";
  import NodeId from "@app/components/NodeId.svelte";
  import ScrollArea from "@app/components/ScrollArea.svelte";
  import Topbar from "@app/components/Topbar.svelte";
  import RawIdentityDocumentModal from "@app/modals/RawIdentityDocument.svelte";

  import Layout from "./Layout.svelte";

  interface Props {
    repo: RepoInfo;
    identity: Identity;
    revision?: string;
  }

  const { repo, identity, revision }: Props = $props();

  const current = $derived(
    identity.revisions.find(r => r.id === identity.current),
  );
  const rootId = $derived(
    identity.revisions[identity.revisions.length - 1]?.id,
  );

  // Which revision is open in the timeline. The card above always shows the
  // document in force; expanding a revision explains it in place rather than
  // swapping what the card is describing. Seeded from the route so a deep
  // link arrives with its revision open, and writable so a click can toggle
  // it without a navigation.
  let expandedId: string | undefined = $derived(revision);

  function toggle(id: string) {
    expandedId = expandedId === id ? undefined : id;
  }
</script>

<style>
  .page {
    display: flex;
    flex-direction: column;
    height: 100%;
    /* The card reads as an object resting on the page rather than as the page
       itself, so the field behind it is a step darker than the card. The two
       themes step in opposite directions: in light the card is already the
       lightest surface, in dark it is one above the base. */
    background-color: var(--color-surface-subtle);
  }
  :global([data-theme="dark"]) .page {
    background-color: var(--color-surface-base);
  }
  .topbar-title {
    font: var(--txt-body-m-semibold);
    color: var(--color-text-secondary);
    padding-right: 0.25rem;
  }
  .topbar-summary {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    margin-left: auto;
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
  }
  .column {
    width: 100%;
    max-width: 44rem;
    margin: 0 auto;
    padding: 2rem 1.5rem 3rem;
    display: flex;
    flex-direction: column;
    gap: 2rem;
  }
  .card {
    background-color: var(--color-surface-canvas);
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-lg);
    box-shadow: var(--elevation-low);
    padding: 1.5rem;
  }
  .card-footer {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.75rem;
    margin-top: 1.5rem;
    padding-top: 1.25rem;
    border-top: 1px solid var(--color-border-subtle);
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
  }
  .history-title {
    font: var(--txt-body-m-medium);
    color: var(--color-text-secondary);
    margin-bottom: 0.75rem;
  }
  .timeline-rail {
    position: relative;
  }
  .timeline {
    position: relative;
  }
  /* The rail runs behind the rows; each icon paints the page colour over it so
     the line reads as connecting the markers. */
  .timeline.has-runs::before {
    content: "";
    position: absolute;
    top: 1.25rem;
    bottom: 1.25rem;
    left: 1rem;
    width: 1px;
    background-color: var(--color-border-mid);
    pointer-events: none;
    z-index: -1;
  }
  .timeline-item {
    display: flex;
    align-items: center;
    gap: 0.625rem;
    width: 100%;
    min-height: 2.5rem;
    padding: 0.375rem 0.5rem;
    border: 0;
    border-radius: var(--border-radius-sm);
    background: none;
    text-align: left;
    color: inherit;
    cursor: pointer;
    font: var(--txt-body-m-regular);
  }
  .timeline-item .icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1rem;
    flex-shrink: 0;
    padding: 0.25rem 0;
    background-color: var(--color-surface-subtle);
    color: var(--color-text-tertiary);
  }
  :global([data-theme="dark"]) .timeline-item .icon {
    background-color: var(--color-surface-base);
  }
  .timeline-item .icon[data-state="accepted"] {
    color: var(--color-feedback-success-text);
  }
  .timeline-item .icon[data-state="rejected"],
  .timeline-item .icon[data-state="redacted"] {
    color: var(--color-feedback-error-text);
  }
  .timeline-item:hover,
  .timeline-item.expanded {
    background-color: var(--color-surface-mid);
  }
  .timeline-item:hover .icon,
  .timeline-item.expanded .icon {
    background-color: var(--color-surface-mid);
  }
  /* Hovering swaps the state marker for an expand affordance, the way the
     patch timeline signals that a row opens. */
  .icon-stack {
    display: grid;
    width: 1rem;
    place-items: center;
  }
  .icon-default,
  .icon-hover {
    grid-area: 1 / 1;
    transition:
      opacity 150ms ease,
      transform 150ms ease;
  }
  .icon-hover {
    opacity: 0;
    transform: rotate(-90deg);
  }
  .timeline-item:hover .icon-default,
  .timeline-item:focus-visible .icon-default {
    opacity: 0;
    transform: rotate(90deg);
  }
  .timeline-item:hover .icon-hover,
  .timeline-item:focus-visible .icon-hover {
    opacity: 1;
    transform: none;
  }
  .item-body {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.5rem;
    min-width: 0;
    flex: 1 1 0;
  }
  .item-title {
    font: var(--txt-body-m-regular);
    color: var(--color-text-primary);
    min-width: 0;
  }
  /* Pushed to the far right by the body's flex, the way the patch timeline
     parks its hash and timestamp. */
  .item-meta {
    display: flex;
    align-items: center;
    gap: 0.625rem;
    flex-shrink: 0;
    font: var(--txt-body-m-regular);
    padding-top: 0.125rem;
  }
  .timestamp {
    color: var(--color-text-quaternary);
  }
  .current-marker {
    color: var(--color-text-brand);
    white-space: nowrap;
  }
  /* Indented to line up with the row's text, clear of the rail marker:
     0.5rem row padding + 1rem icon + 0.625rem gap. */
  .detail {
    min-width: 0;
    padding: 0.25rem 0.5rem 1rem 2.125rem;
  }
  .detail-section + .detail-section {
    margin-top: 1.25rem;
  }
  .detail-title {
    font: var(--txt-body-m-medium);
    color: var(--color-text-secondary);
    margin-bottom: 0.625rem;
  }
  .state-caption {
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
    margin-bottom: 1rem;
  }
  .signatures {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
  }
  .signature {
    flex: 1 1 11rem;
    min-width: 0;
    padding: 0.75rem 0.875rem 0.625rem;
    border-radius: var(--border-radius-sm);
    background-color: var(--color-surface-canvas);
  }
  .signature-name {
    font: var(--txt-body-l-regular);
    font-style: italic;
    color: var(--color-text-primary);
    padding-bottom: 0.375rem;
    overflow-wrap: anywhere;
  }
  .signature-name.struck {
    color: var(--color-text-tertiary);
    text-decoration: line-through;
  }
  .signature-rule {
    height: 1px;
    background-color: var(--color-border-mid);
  }
  .signature-foot {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    margin-top: 0.5rem;
    font: var(--txt-body-s-regular);
    color: var(--color-text-secondary);
  }
  .accepted-icon {
    color: var(--color-feedback-success-text);
    display: inline-flex;
  }
  .rejected-icon {
    color: var(--color-feedback-error-text);
    display: inline-flex;
  }
  .muted {
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
  }
  .quorum-note {
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
    margin-top: 0.75rem;
  }
  .detail-foot {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.75rem;
    margin-top: 1.25rem;
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
  }
  .raw-button {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.25rem 0.625rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    background-color: var(--color-surface-canvas);
    cursor: pointer;
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
  }
  .raw-button:hover,
  .raw-button:focus-visible {
    background-color: var(--color-surface-subtle);
    color: var(--color-text-primary);
  }
</style>

<Layout selfScroll>
  <div class="page">
    <Topbar>
      <span class="topbar-title">Identity</span>
      <div class="topbar-summary">
        <Icon name="revision" />
        {identity.revisions.length}
        {pluralize("revision", identity.revisions.length)}
      </div>
    </Topbar>

    <ScrollArea style="flex: 1; min-height: 0;">
      <div class="column">
        <div class="card">
          <IdentityDocument
            doc={identity.doc}
            rid={identity.rid}
            revision={identity.current}
            withHeader />
          <div class="card-footer">
            <span>
              Revision <Id
                id={identity.current}
                clipboard={identity.current}
                label="revision ID" />
            </span>
            {#if current}
              <span>
                in force since {absoluteTimestamp(current.timestamp)}
              </span>
            {/if}
          </div>
        </div>

        <div>
          <div class="history-title">History</div>
          <div class="timeline-rail">
            <div
              class="timeline"
              class:has-runs={identity.revisions.length > 1}>
              {#each identity.revisions as rev (rev.id)}
                {@const open = expandedId === rev.id}
                <div class="timeline-entry">
                  <button
                    type="button"
                    class="timeline-item"
                    class:expanded={open}
                    aria-expanded={open}
                    onclick={() => toggle(rev.id)}>
                    <span class="icon" data-state={rev.state.status}>
                      <span class="icon-stack">
                        <span class="icon-default">
                          <Icon name={stateIcon(rev.state)} />
                        </span>
                        <span class="icon-hover">
                          <Icon
                            name={open
                              ? "collapse-vertical"
                              : "expand-vertical"} />
                        </span>
                      </span>
                    </span>
                    <span class="item-body">
                      <NodeId {...authorForNodeId(rev.author)} />
                      <span class="item-title txt-overflow">{rev.title}</span>
                    </span>
                    <span class="item-meta">
                      {#if rev.id === identity.current}
                        <span class="current-marker">Current</span>
                      {/if}
                      <span
                        class="timestamp"
                        title={absoluteTimestamp(rev.timestamp)}>
                        {formatTimestamp(rev.timestamp)}
                      </span>
                    </span>
                  </button>
                  {#if open}
                    <div class="detail" transition:slide={{ duration: 180 }}>
                      {#if stateCaption(rev.state)}
                        <div class="state-caption">
                          {stateCaption(rev.state)}
                        </div>
                      {/if}

                      {#if rev.description}
                        <div class="detail-section">
                          <div class="detail-title">Description</div>
                          <Markdown
                            breaks
                            content={rev.description}
                            rid={repo.rid} />
                        </div>
                      {/if}

                      <div class="detail-section">
                        <div class="detail-title">Changes</div>
                        <IdentityChanges
                          changes={rev.changes}
                          root={rev.id === rootId} />
                      </div>

                      <div class="detail-section">
                        <div class="detail-title">Signatures</div>
                        {#if rev.accepted.length > 0 || rev.rejected.length > 0}
                          <div class="signatures">
                            {#each rev.accepted as delegate (delegate.did)}
                              <div class="signature">
                                <div class="signature-name">
                                  {delegate.alias ?? delegate.did.slice(8, 20)}
                                </div>
                                <div class="signature-rule"></div>
                                <div class="signature-foot">
                                  <span class="accepted-icon">
                                    <Icon name="checkmark" />
                                  </span>
                                  <span>signed</span>
                                </div>
                              </div>
                            {/each}
                            {#each rev.rejected as delegate (delegate.did)}
                              <div class="signature">
                                <div class="signature-name struck">
                                  {delegate.alias ?? delegate.did.slice(8, 20)}
                                </div>
                                <div class="signature-rule"></div>
                                <div class="signature-foot">
                                  <span class="rejected-icon">
                                    <Icon name="close" />
                                  </span>
                                  <span>rejected</span>
                                </div>
                              </div>
                            {/each}
                          </div>
                        {:else}
                          <div class="muted">No signatures recorded.</div>
                        {/if}
                        <div class="quorum-note">
                          {#if rev.quorum}
                            Reached quorum: a majority of the delegates in force
                            at the time signed it.
                          {:else}
                            Did not reach quorum: a majority of the delegates in
                            force at the time did not sign it.
                          {/if}
                        </div>
                      </div>

                      <div class="detail-foot">
                        <IdentityStateBadge state={rev.state} />
                        <Id
                          id={rev.id}
                          clipboard={rev.id}
                          label="revision ID" />
                        <span>{absoluteTimestamp(rev.timestamp)}</span>
                        <button
                          type="button"
                          class="raw-button"
                          onclick={() =>
                            show({
                              component: RawIdentityDocumentModal,
                              props: { raw: rev.doc.raw, revision: rev.id },
                            })}>
                          <Icon name="code" />
                          Document at this revision
                        </button>
                      </div>
                    </div>
                  {/if}
                </div>
              {/each}
            </div>
          </div>
        </div>
      </div>
    </ScrollArea>
  </div>
</Layout>
