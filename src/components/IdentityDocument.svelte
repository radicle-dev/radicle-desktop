<script lang="ts">
  import type { Doc } from "@bindings/identity/Doc";

  import { show } from "@app/lib/modal";
  import { authorForNodeId, pluralize, truncateDid } from "@app/lib/utils";

  import Button from "@app/components/Button.svelte";
  import HoverPopover from "@app/components/HoverPopover.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import NodeId from "@app/components/NodeId.svelte";
  import UserAvatar from "@app/components/UserAvatar.svelte";
  import VisibilityBadge from "@app/components/VisibilityBadge.svelte";
  import RawIdentityDocumentModal from "@app/modals/RawIdentityDocument.svelte";

  interface Props {
    doc: Doc;
    rid?: string;
    // Shown alongside the raw document so a copied blob is traceable.
    revision?: string;
  }

  const { doc, rid, revision }: Props = $props();

  // Payloads the sections below render in full. Anything else is listed by
  // name so an unmodelled payload is never silently dropped.
  const knownPayloads = ["xyz.radicle.project", "xyz.radicle.crefs"];
  const otherPayloads = $derived(
    doc.payloadIds.filter(id => !knownPayloads.includes(id)),
  );
</script>

<style>
  .details {
    min-width: 0;
  }
  .section {
    padding: 0.875rem 0;
  }
  .section + .section {
    border-top: 1px solid var(--color-border-subtle);
  }
  .section-title {
    font: var(--txt-body-m-medium);
    color: var(--color-text-primary);
    margin: 0 0 0.625rem;
  }
  /* Avatars grouped, then names, then the rule they operate under: one
     horizontal block rather than a row per delegate. */
  .delegates {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.75rem 1rem;
  }
  .delegate-cards {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.5rem;
    min-width: 0;
  }
  /* Same bordered chip the patch metadata uses for an author. */
  .delegate-card {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    height: 2rem;
    padding: 0 0.625rem 0 0.375rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    background-color: var(--color-surface-canvas);
    font: var(--txt-body-m-regular);
    color: var(--color-text-primary);
  }
  .delegate-card:hover {
    background-color: var(--color-surface-subtle);
  }
  .delegate-alias {
    min-width: 0;
    overflow-wrap: anywhere;
  }
  .avatar {
    width: 1.25rem;
    height: 1.25rem;
    overflow: hidden;
    border-radius: 2px;
    flex-shrink: 0;
  }
  .avatar :global(img) {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .delegates-head {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
    margin-bottom: 0.625rem;
  }
  .delegates-head .section-title {
    margin: 0;
  }
  .quorum {
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
    white-space: nowrap;
  }
  .tip-alias {
    font: var(--txt-body-m-regular);
    color: var(--color-text-primary);
    white-space: nowrap;
  }
  .tip-did {
    font: var(--txt-code-small);
    color: var(--color-text-tertiary);
    white-space: nowrap;
  }
  /* Label beside value, both sized to content: a details list, not rows
     spanning the window. */
  .fields {
    display: grid;
    grid-template-columns: max-content minmax(0, 1fr);
    column-gap: 2rem;
    row-gap: 0.5rem;
    align-items: baseline;
    font: var(--txt-body-m-regular);
  }
  .field-label {
    color: var(--color-text-secondary);
  }
  .field-value {
    min-width: 0;
    color: var(--color-text-primary);
    overflow-wrap: anywhere;
    justify-self: start;
  }
  .section-action {
    margin-top: 0.875rem;
  }
  .mono {
    font: var(--txt-code-regular);
  }
  .hint {
    color: var(--color-text-secondary);
  }
  .empty {
    color: var(--color-text-secondary);
  }
  .inline-list {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
  }
  .rule + .rule {
    margin-top: 0.625rem;
  }
  .rule-pattern {
    font: var(--txt-code-regular);
    color: var(--color-text-primary);
    overflow-wrap: anywhere;
  }
  .rule-detail {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.375rem;
    margin-top: 0.125rem;
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
  }
</style>

<div class="details">
  <div class="section">
    <div class="delegates-head">
      <h2 class="section-title">Delegates</h2>
      <span class="quorum">
        {doc.majority} of {doc.delegates.length} must sign off
      </span>
    </div>
    <div class="delegates">
      <div class="delegate-cards">
        {#each doc.delegates as delegate (delegate.did)}
          <HoverPopover
            placement="bottom-start"
            stylePadding="0.375rem 0.625rem">
            {#snippet toggle()}
              <span class="delegate-card">
                <span class="avatar">
                  <UserAvatar nodeId={delegate.did} styleWidth="1.25rem" />
                </span>
                <span class="delegate-alias">
                  {delegate.alias ?? truncateDid(delegate.did)}
                </span>
              </span>
            {/snippet}
            {#snippet popover()}
              <div>
                <div class="tip-alias">
                  {delegate.alias ?? truncateDid(delegate.did)}
                </div>
                <div class="tip-did">{truncateDid(delegate.did)}</div>
              </div>
            {/snippet}
          </HoverPopover>
        {/each}
      </div>
    </div>
  </div>

  <div class="section">
    <h2 class="section-title">Document</h2>
    <div class="fields">
      {#if doc.project}
        <span class="field-label">Default branch</span>
        <span class="field-value mono">{doc.project.defaultBranch}</span>
      {/if}
      <span class="field-label">Visibility</span>
      <span class="field-value">
        <VisibilityBadge type={doc.visibility.type} />
      </span>
      {#if doc.visibility.type === "private" && doc.visibility.allow}
        <span class="field-label">Also visible to</span>
        <span class="field-value inline-list">
          {#each doc.visibility.allow as peer (peer.did)}
            <NodeId {...authorForNodeId(peer)} />
          {/each}
        </span>
      {/if}
      <span class="field-label">Signature threshold</span>
      <span class="field-value">
        {doc.threshold} of {doc.delegates.length}
        <span class="hint">
          — delegate signatures that make a ref canonical
        </span>
      </span>
      {#if rid}
        <span class="field-label">Repository ID</span>
        <span class="field-value mono">
          <Id id={rid} clipboard={rid} label="repository ID" shorten={false} />
        </span>
      {/if}
      <span class="field-label">Document version</span>
      <span class="field-value">{doc.version}</span>
    </div>
    <div class="section-action">
      <Button
        styleHeight="2rem"
        variant="outline"
        onclick={() =>
          show({
            component: RawIdentityDocumentModal,
            props: { raw: doc.raw, revision },
          })}>
        <Icon name="code" />View raw document
      </Button>
    </div>
  </div>

  <div class="section">
    <h2 class="section-title">Canonical refs</h2>
    {#if doc.canonicalRefs.length > 0}
      {#each doc.canonicalRefs as rule (rule.pattern)}
        <div class="rule">
          <div class="rule-pattern">{rule.pattern}</div>
          <div class="rule-detail">
            {#if rule.delegates}
              <span>
                Needs {rule.threshold ?? doc.threshold}
                {pluralize("signature", rule.threshold ?? doc.threshold)}
                from the delegate set
              </span>
            {:else if rule.allow.length > 0}
              <span>
                Needs {rule.threshold ?? rule.allow.length}
                {pluralize("signature", rule.threshold ?? rule.allow.length)}
                from
              </span>
              {#each rule.allow as peer (peer.did)}
                <NodeId {...authorForNodeId(peer)} />
              {/each}
            {:else if rule.threshold !== null}
              <span>
                Needs {rule.threshold}
                {pluralize("signature", rule.threshold)}
              </span>
            {/if}
          </div>
        </div>
      {/each}
    {:else}
      <div class="empty">
        No explicit rules. The default branch is resolved by delegate quorum.
      </div>
    {/if}
  </div>

  {#if otherPayloads.length > 0}
    <div class="section">
      <h2 class="section-title">Other payloads</h2>
      {#each otherPayloads as payload (payload)}
        <div class="rule">
          <div class="rule-pattern">{payload}</div>
          <div class="rule-detail">
            Not rendered by this app. See the raw document.
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
