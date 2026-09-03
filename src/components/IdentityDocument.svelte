<script lang="ts">
  import type { Doc } from "@bindings/identity/Doc";

  import { show } from "@app/lib/modal";
  import { authorForNodeId, pluralize, truncateDid } from "@app/lib/utils";

  import HoverPopover from "@app/components/HoverPopover.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import NodeId from "@app/components/NodeId.svelte";
  import RepoAvatar from "@app/components/RepoAvatar.svelte";
  import UserAvatar from "@app/components/UserAvatar.svelte";
  import VisibilityBadge from "@app/components/VisibilityBadge.svelte";
  import RawIdentityDocumentModal from "@app/modals/RawIdentityDocument.svelte";

  interface Props {
    doc: Doc;
    rid?: string;
    // Promote name, description and visibility into a card head instead of
    // listing them as fields. Used where the document is presented as a card.
    withHeader?: boolean;
    // Shown alongside the raw document so a copied blob is traceable.
    revision?: string;
  }

  const { doc, rid, withHeader = false, revision }: Props = $props();

  // Payloads the sections below render in full. Anything else is listed by
  // name so an unmodelled payload is never silently dropped.
  const knownPayloads = ["xyz.radicle.project", "xyz.radicle.crefs"];
  const otherPayloads = $derived(
    doc.payloadIds.filter(id => !knownPayloads.includes(id)),
  );

  // The quorum meter fills one pip per signature required, out of one pip per
  // delegate, so "2 of 3" is legible without reading the sentence.
  const pips = $derived(
    Array.from({ length: doc.delegates.length }, (_, i) => i < doc.majority),
  );
</script>

<style>
  .sections {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }
  .sections.spaced {
    padding-top: 1.25rem;
  }
  .card-head {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 0.375rem;
    padding-bottom: 0.5rem;
  }
  .card-avatar {
    width: 4.5rem;
    height: 4.5rem;
    overflow: hidden;
    border-radius: var(--border-radius-md);
    margin-bottom: 0.625rem;
    flex-shrink: 0;
  }
  .card-avatar :global(img) {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .card-name {
    font: var(--txt-heading-s);
    color: var(--color-text-primary);
    overflow-wrap: anywhere;
  }
  .card-description {
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
    max-width: 30rem;
  }
  .card-badges {
    margin-top: 0.375rem;
  }
  /* Every delegate is drawn: the row wraps rather than collapsing into a
     "+N more". */
  .delegate-stack {
    display: flex;
    justify-content: center;
    flex-wrap: wrap;
    gap: 0.5rem;
  }
  .stack-avatar {
    width: 2.75rem;
    height: 2.75rem;
    overflow: hidden;
    border-radius: var(--border-radius-sm);
  }
  .stack-avatar :global(img) {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .stack-tip {
    white-space: nowrap;
    text-align: left;
  }
  .stack-tip-alias {
    font: var(--txt-body-m-regular);
    color: var(--color-text-primary);
  }
  .stack-tip-did {
    font: var(--txt-code-small);
    color: var(--color-text-tertiary);
  }
  .quorum-meter {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-top: 0.5rem;
  }
  .allow-list {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }
  .pips {
    display: flex;
    gap: 0.1875rem;
  }
  .pip {
    width: 1rem;
    height: 0.375rem;
    border-radius: 999px;
    background-color: var(--color-surface-mid);
  }
  .pip.filled {
    background-color: var(--color-surface-brand-primary);
  }
  .quorum-count {
    font: var(--txt-body-s-regular);
    color: var(--color-text-secondary);
    white-space: nowrap;
  }
  .section-title {
    font: var(--txt-body-m-medium);
    color: var(--color-text-secondary);
    margin-bottom: 0.75rem;
  }
  .bento {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.5rem;
  }
  .tile {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    min-width: 0;
    padding: 1rem 0.875rem;
    border-radius: var(--border-radius-md);
    background-color: var(--color-surface-subtle);
  }
  /* Facts that need the room — a RID, a description, a threshold with its
     caption — take the full width instead of being squeezed into a column. */
  .tile.wide {
    grid-column: 1 / -1;
  }
  .tile-label {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.375rem;
    font: var(--txt-body-s-regular);
    color: var(--color-text-tertiary);
    margin-top: 0.375rem;
  }
  .tile-value {
    font: var(--txt-heading-l);
    color: var(--color-text-primary);
    overflow-wrap: anywhere;
    max-width: 100%;
  }
  .tile-value.mono {
    font: var(--txt-code-regular);
    font-size: 1.125rem;
    line-height: 1.5rem;
  }
  .tile-value :global(.txt-id) {
    color: var(--color-text-primary);
    font: var(--txt-code-regular);
    font-size: 1.125rem;
    line-height: 1.5rem;
  }
  .allow-list {
    align-items: center;
  }
  .tile-hint {
    font: var(--txt-body-s-regular);
    color: var(--color-text-secondary);
    margin-top: 0.375rem;
    max-width: 24rem;
  }
  .mono {
    font: var(--txt-code-regular);
  }
  /* Plain rows on the card, divided by a rule rather than boxed: they are a
     list of statements, not facts to be scanned like the tiles. */
  .rule {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
    padding: 0.75rem 0;
  }
  .rule:first-child {
    padding-top: 0;
  }
  .rule:last-child {
    padding-bottom: 0;
  }
  .rule + .rule {
    border-top: 1px solid var(--color-border-subtle);
  }
  .rules {
    display: flex;
    flex-direction: column;
  }
  .rule-pattern {
    font: var(--txt-code-regular);
    color: var(--color-text-primary);
  }
  .rule-detail {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.5rem;
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
  }
  .empty {
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
  }
  .raw-button {
    display: flex;
    width: 100%;
    justify-content: center;
    align-items: center;
    gap: 0.375rem;
    padding: 0.375rem 0.75rem;
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

{#if withHeader}
  <div class="card-head">
    {#if rid}
      <div class="card-avatar">
        <RepoAvatar name={doc.project?.name ?? ""} {rid} styleWidth="4.5rem" />
      </div>
    {/if}
    <div class="card-name">
      {doc.project?.name ?? "Identity document"}
    </div>
    {#if doc.project?.description}
      <div class="card-description">{doc.project.description}</div>
    {/if}
    <div class="card-badges">
      <VisibilityBadge type={doc.visibility.type} />
    </div>
  </div>
{/if}

<div class="sections" class:spaced={withHeader}>
  <div>
    <div class="bento">
      <div class="tile wide">
        <div class="delegate-stack">
          {#each doc.delegates as delegate (delegate.did)}
            <HoverPopover placement="bottom" stylePadding="0.375rem 0.625rem">
              {#snippet toggle()}
                <div class="stack-avatar">
                  <UserAvatar nodeId={delegate.did} styleWidth="2.75rem" />
                </div>
              {/snippet}
              {#snippet popover()}
                <div class="stack-tip">
                  <div class="stack-tip-alias">
                    {delegate.alias ?? truncateDid(delegate.did)}
                  </div>
                  {#if delegate.alias}
                    <div class="stack-tip-did">{truncateDid(delegate.did)}</div>
                  {/if}
                </div>
              {/snippet}
            </HoverPopover>
          {/each}
        </div>
        <div class="tile-label"><Icon name="badge" />Delegates</div>
        <div
          class="quorum-meter"
          title="{doc.majority} of {doc.delegates
            .length} delegates must sign off on a change">
          <div class="pips">
            {#each pips as filled, i (i)}
              <span class="pip" class:filled></span>
            {/each}
          </div>
          <span class="quorum-count">
            {doc.majority}/{doc.delegates.length} to sign off
          </span>
        </div>
      </div>

      {#if !withHeader && doc.project}
        <div class="tile">
          <div class="tile-value">{doc.project.name}</div>

          <div class="tile-label">Name</div>
        </div>
        <div class="tile">
          <div class="tile-value">
            <VisibilityBadge type={doc.visibility.type} />
          </div>

          <div class="tile-label">Visibility</div>
        </div>
        <div class="tile wide">
          <div class="tile-value">
            {#if doc.project.description}
              {doc.project.description}
            {:else}
              <span class="empty">None</span>
            {/if}
          </div>

          <div class="tile-label">Description</div>
        </div>
      {/if}

      {#if doc.project}
        <div class="tile">
          <div class="tile-value mono">{doc.project.defaultBranch}</div>

          <div class="tile-label">
            <Icon name="branch" />Default branch
          </div>
        </div>
      {/if}

      <div class="tile">
        <div class="tile-value">{doc.version}</div>

        <div class="tile-label"><Icon name="hash" />Document version</div>
      </div>

      <div class="tile wide">
        <div class="tile-value">
          {doc.threshold} of {doc.delegates.length}
        </div>

        <div class="tile-label">
          <Icon name="badge" />Signature threshold
        </div>
        <div class="tile-hint">
          Delegate signatures that make a ref canonical.
        </div>
      </div>

      {#if rid}
        <div class="tile wide">
          <div class="tile-value mono">
            <Id
              id={rid}
              clipboard={rid}
              label="repository ID"
              shorten={false} />
          </div>

          <div class="tile-label">
            <Icon name="repository" />Repository ID
          </div>
        </div>
      {/if}

      {#if doc.visibility.type === "private" && doc.visibility.allow}
        <div class="tile wide">
          <div class="allow-list">
            {#each doc.visibility.allow as peer (peer.did)}
              <NodeId {...authorForNodeId(peer)} />
            {/each}
          </div>
          <div class="tile-label"><Icon name="eye" />Also visible to</div>
        </div>
      {/if}
    </div>
  </div>

  <div>
    <button
      type="button"
      class="raw-button"
      onclick={() =>
        show({
          component: RawIdentityDocumentModal,
          props: { raw: doc.raw, revision },
        })}>
      <Icon name="code" />
      View raw document
    </button>
  </div>

  <div>
    <div class="section-title">Canonical refs</div>
    {#if doc.canonicalRefs.length > 0}
      <div class="rules">
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
      </div>
    {:else}
      <div class="empty">
        No explicit rules. The default branch is resolved by delegate quorum.
      </div>
    {/if}
  </div>

  {#if otherPayloads.length > 0}
    <div>
      <div class="section-title">Other payloads</div>
      <div class="rules">
        {#each otherPayloads as payload (payload)}
          <div class="rule">
            <div class="rule-pattern">{payload}</div>
            <div class="rule-detail">
              Not rendered by this app. See the raw document below.
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>
