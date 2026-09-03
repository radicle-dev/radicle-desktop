<script lang="ts">
  import type { Config } from "@bindings/config/Config";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import { explorerUrl, truncateDid } from "@app/lib/utils";

  import CheckoutRepoButton from "@app/components/CheckoutRepoButton.svelte";
  import HoverPopover from "@app/components/HoverPopover.svelte";
  import Icon from "@app/components/Icon.svelte";
  import ShareButton from "@app/components/ShareButton.svelte";
  import UserAvatar from "@app/components/UserAvatar.svelte";
  import VisibilityBadge from "@app/components/VisibilityBadge.svelte";

  interface Props {
    repo: RepoInfo;
    config: Config;
  }

  const { repo, config }: Props = $props();

  const project = $derived(repo.payloads["xyz.radicle.project"]!);
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
  .meta {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-left: auto;
    flex-shrink: 0;
  }
  .meta-item {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    font: var(--txt-body-m-regular);
  }
  .meta-label {
    color: var(--color-text-secondary);
  }
  .meta-value {
    color: var(--color-text-primary);
  }
  .avatars {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }
  .avatar-wrap {
    width: 1.25rem;
    height: 1.25rem;
    overflow: hidden;
    flex-shrink: 0;
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-shrink: 0;
  }
  a {
    color: inherit;
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    text-decoration: none;
    color: var(--color-text-secondary);
  }
  a:hover {
    color: var(--color-text-primary);
  }
</style>

<div class="header">
  <div class="project txt-selectable">
    <div class="name txt-overflow">{project.data.name}</div>
    {#if project.data.description}
      <div class="description txt-overflow">{project.data.description}</div>
    {/if}
  </div>

  <div class="meta">
    <VisibilityBadge type={repo.visibility.type} />

    <div class="meta-item">
      <span class="meta-label">Delegates</span>
      <span class="meta-value">{repo.threshold}/{repo.delegates.length}</span>
      <div class="avatars">
        {#each repo.delegates as delegate}
          <HoverPopover placement="bottom-start" stylePadding="0.25rem 0.5rem">
            {#snippet toggle()}
              <div class="avatar-wrap">
                <UserAvatar nodeId={delegate.did} styleWidth="1.25rem" />
              </div>
            {/snippet}
            {#snippet popover()}
              <a
                class="global-flex txt-body-m-regular"
                style:white-space="nowrap"
                style:text-decoration="none"
                style:width="100%"
                href={explorerUrl(`users/${delegate.did}`, config)}
                target="_blank">
                {#if delegate.alias}
                  <span class="txt-overflow alias">
                    {delegate.alias}
                  </span>
                {:else}
                  <span class="no-alias">
                    {truncateDid(delegate.did)}
                  </span>
                {/if}
                <span style:margin-left="auto">
                  <Icon name="open-external" />
                </span>
              </a>
            {/snippet}
          </HoverPopover>
        {/each}
      </div>
    </div>
  </div>

  <div class="actions">
    <ShareButton
      explorerPath={repo.rid}
      id={repo.rid}
      idLabel="repository"
      {config} />
    <CheckoutRepoButton rid={repo.rid} />
  </div>
</div>
