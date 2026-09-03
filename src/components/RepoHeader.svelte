<script lang="ts">
  import type { Config } from "@bindings/config/Config";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import * as router from "@app/lib/router";

  import CheckoutRepoButton from "@app/components/CheckoutRepoButton.svelte";
  import ShareButton from "@app/components/ShareButton.svelte";
  import UserAvatar from "@app/components/UserAvatar.svelte";
  import VisibilityBadge from "@app/components/VisibilityBadge.svelte";

  interface Props {
    repo: RepoInfo;
    config: Config;
  }

  const { repo, config }: Props = $props();

  const project = $derived(repo.payloads["xyz.radicle.project"]!);

  // A stack shows a few faces and counts the rest, so a repo with many
  // delegates does not push the rest of the header around.
  const SHOWN = 3;
  const shown = $derived(repo.delegates.slice(0, SHOWN));
  const overflow = $derived(repo.delegates.length - shown.length);
</script>

<style>
  .header {
    display: flex;
    align-items: center;
    flex-direction: row;
    gap: 1rem;
    padding: 0.625rem 1rem;
    flex-shrink: 0;
  }
  .project {
    flex: 1;
    min-width: 0;
  }
  .name {
    font: var(--txt-body-l-semibold);
    color: var(--color-text-primary);
  }
  .description {
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
  }
  .identity-button {
    display: flex;
    align-items: center;
    gap: 0.625rem;
    margin-left: auto;
    flex-shrink: 0;
    height: 2rem;
    padding: 0 0.625rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    background-color: var(--color-surface-canvas);
    cursor: pointer;
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
  }
  .identity-button:hover,
  .identity-button:focus-visible {
    background-color: var(--color-surface-subtle);
    color: var(--color-text-primary);
  }
  .delegates {
    display: flex;
    align-items: center;
    gap: 0.375rem;
  }
  .delegate-count {
    color: inherit;
  }
  .avatars {
    display: flex;
    align-items: center;
  }
  /* Overlapped, each face ringed in the control's own colour so the stack
     reads as a stack rather than as smudged tiles. */
  .avatar-wrap + .avatar-wrap {
    margin-left: -0.4375rem;
  }
  .avatar-wrap {
    width: 1.25rem;
    height: 1.25rem;
    overflow: hidden;
    border-radius: 2px;
    flex-shrink: 0;
    box-shadow: 0 0 0 1.5px var(--color-surface-canvas);
  }
  .identity-button:hover .avatar-wrap,
  .identity-button:focus-visible .avatar-wrap {
    box-shadow: 0 0 0 1.5px var(--color-surface-subtle);
  }
  .overflow {
    margin-left: 0.375rem;
    font: var(--txt-body-s-regular);
    color: var(--color-text-tertiary);
  }
  .avatar-wrap :global(img) {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-shrink: 0;
  }
</style>

<div class="header">
  <div class="project txt-selectable">
    <div class="name txt-overflow">{project.data.name}</div>
    {#if project.data.description}
      <div class="description txt-overflow">{project.data.description}</div>
    {/if}
  </div>

  <!-- Visibility, the delegate set and the way in to the identity document
       are all facets of the same thing, so they are one control. -->
  <button
    class="identity-button"
    aria-label="View identity document"
    title="View identity document"
    onclick={() => router.push({ resource: "repo.identity", rid: repo.rid })}>
    <VisibilityBadge type={repo.visibility.type} />
    <span class="delegates">
      <span class="avatars">
        {#each shown as delegate (delegate.did)}
          <span class="avatar-wrap">
            <UserAvatar nodeId={delegate.did} styleWidth="1.25rem" />
          </span>
        {/each}
      </span>
      {#if overflow > 0}
        <span class="overflow">+{overflow}</span>
      {/if}
      <span class="delegate-count">
        {repo.threshold}/{repo.delegates.length}
      </span>
    </span>
  </button>

  <div class="actions">
    <ShareButton
      explorerPath={repo.rid}
      id={repo.rid}
      idLabel="repository"
      {config} />
    <CheckoutRepoButton rid={repo.rid} />
  </div>
</div>
