<script lang="ts">
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import debounce from "lodash/debounce";

  import { writeToClipboard } from "@app/lib/invoke";
  import { explorerUrl, truncateDid } from "@app/lib/utils";

  import Button from "@app/components/Button.svelte";
  import CheckoutRepoButton from "@app/components/CheckoutRepoButton.svelte";
  import HoverPopover from "@app/components/HoverPopover.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import JobCob from "@app/components/JobCob.svelte";
  import UserAvatar from "@app/components/UserAvatar.svelte";
  import VisibilityBadge from "@app/components/VisibilityBadge.svelte";

  interface Props {
    repo: RepoInfo;
  }

  const { repo }: Props = $props();

  const project = $derived(repo.payloads["xyz.radicle.project"]!);

  let copyIcon: "link" | "checkmark" = $state("link");
  const restoreCopyIcon = debounce(() => {
    copyIcon = "link";
  }, 1000);

  async function copyLink() {
    await writeToClipboard(explorerUrl(repo.rid));
    copyIcon = "checkmark";
    restoreCopyIcon();
  }
</script>

<style>
  .header {
    display: flex;
    align-items: center;
    flex-direction: row;
    gap: 1rem;
    padding: 0.625rem 1rem;
    border-bottom: 1px solid var(--color-border-subtle);
    flex-shrink: 0;
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
  <div class="txt-selectable">
    <div class="name txt-overflow">{project.data.name}</div>
    {#if project.data.description}
      <div class="description txt-overflow">{project.data.description}</div>
    {/if}
  </div>

  <div class="meta">
    <VisibilityBadge type={repo.visibility.type} />

    <div class="meta-item">
      <span class="meta-label">{project.data.defaultBranch}</span>
      <Icon name="arrow-right" />
      <Id
        id={project.meta.head}
        clipboard={project.meta.head}
        placement="bottom-start" />
      <JobCob rid={repo.rid} commit={project.meta.head} />
    </div>

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
                href={explorerUrl(`users/${delegate.did}`)}
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
    <Button variant="ghost" styleHeight="2rem" onclick={copyLink}>
      <Icon name={copyIcon} />Copy Link
    </Button>
    <CheckoutRepoButton rid={repo.rid} />
  </div>
</div>
