<script lang="ts">
  import type { Doc } from "@bindings/identity/Doc";

  import { authorForNodeId, pluralize, truncateDid } from "@app/lib/utils";

  import Id from "@app/components/Id.svelte";
  import NodeId from "@app/components/NodeId.svelte";
  import VisibilityBadge from "@app/components/VisibilityBadge.svelte";

  interface Props {
    doc: Doc;
    rid?: string;
  }

  const { doc, rid }: Props = $props();

  // Payloads the sections below render in full. Anything else is listed by
  // name so an unmodelled payload is never silently dropped.
  const knownPayloads = ["xyz.radicle.project", "xyz.radicle.crefs"];
  const otherPayloads = $derived(
    doc.payloadIds.filter(id => !knownPayloads.includes(id)),
  );
</script>

<style>
  .section {
    border-bottom: 1px solid var(--color-border-subtle);
  }
  .section-head {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1rem 0.5rem;
  }
  .section-title {
    font: var(--txt-body-m-medium);
    color: var(--color-text-primary);
    margin: 0;
  }
  .section-note {
    margin-left: auto;
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
  }
  .row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    min-height: 2rem;
    padding: 0.375rem 1rem;
    font: var(--txt-body-m-regular);
  }
  .row + .row {
    border-top: 1px solid var(--color-border-subtle);
  }
  .row-label {
    flex: 0 0 11rem;
    color: var(--color-text-secondary);
  }
  .row-value {
    min-width: 0;
    color: var(--color-text-primary);
    overflow-wrap: anywhere;
  }
  .row-aside {
    margin-left: auto;
    flex-shrink: 0;
    font: var(--txt-code-small);
    color: var(--color-text-tertiary);
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
  .allow-list {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.75rem;
    min-width: 0;
  }
</style>

<div class="section">
  <div class="section-head">
    <h2 class="section-title">Delegates</h2>
    <span class="section-note">
      {doc.majority} of {doc.delegates.length}
      {pluralize("delegate", doc.delegates.length)} must sign off on a change
    </span>
  </div>
  {#each doc.delegates as delegate (delegate.did)}
    <div class="row">
      <NodeId {...authorForNodeId(delegate)} />
      <span class="row-aside">{truncateDid(delegate.did)}</span>
    </div>
  {/each}
</div>

<div class="section">
  <div class="section-head">
    <h2 class="section-title">Document</h2>
  </div>
  {#if doc.project}
    <div class="row">
      <span class="row-label">Default branch</span>
      <span class="row-value mono">{doc.project.defaultBranch}</span>
    </div>
  {/if}
  <div class="row">
    <span class="row-label">Visibility</span>
    <span class="row-value">
      <VisibilityBadge type={doc.visibility.type} />
    </span>
  </div>
  {#if doc.visibility.type === "private" && doc.visibility.allow}
    <div class="row">
      <span class="row-label">Also visible to</span>
      <span class="row-value allow-list">
        {#each doc.visibility.allow as peer (peer.did)}
          <NodeId {...authorForNodeId(peer)} />
        {/each}
      </span>
    </div>
  {/if}
  <div class="row">
    <span class="row-label">Signature threshold</span>
    <span class="row-value">
      {doc.threshold} of {doc.delegates.length}
      <span class="hint">— delegate signatures that make a ref canonical</span>
    </span>
  </div>
  {#if rid}
    <div class="row">
      <span class="row-label">Repository ID</span>
      <span class="row-value mono">
        <Id id={rid} clipboard={rid} label="repository ID" shorten={false} />
      </span>
    </div>
  {/if}
  <div class="row">
    <span class="row-label">Document version</span>
    <span class="row-value">{doc.version}</span>
  </div>
</div>

<div class="section">
  <div class="section-head">
    <h2 class="section-title">Canonical refs</h2>
  </div>
  {#if doc.canonicalRefs.length > 0}
    {#each doc.canonicalRefs as rule (rule.pattern)}
      <div class="row">
        <span class="row-label mono">{rule.pattern}</span>
        <span class="row-value allow-list">
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
        </span>
      </div>
    {/each}
  {:else}
    <div class="row">
      <span class="row-value empty">
        No explicit rules. The default branch is resolved by delegate quorum.
      </span>
    </div>
  {/if}
</div>

{#if otherPayloads.length > 0}
  <div class="section">
    <div class="section-head">
      <h2 class="section-title">Other payloads</h2>
    </div>
    {#each otherPayloads as payload (payload)}
      <div class="row">
        <span class="row-label mono">{payload}</span>
        <span class="row-value hint">
          Not rendered by this app. See the raw document.
        </span>
      </div>
    {/each}
  </div>
{/if}
