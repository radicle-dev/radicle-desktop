<script lang="ts">
  import type { Config } from "@bindings/config/Config";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";
  import type { RepoTeam } from "@bindings/repo/RepoTeam";

  import { invoke } from "@app/lib/invoke";
  import { explorerUrl, formatRepositoryId, truncateDid } from "@app/lib/utils";

  import CheckoutRepoButton from "@app/components/CheckoutRepoButton.svelte";
  import HoverPopover from "@app/components/HoverPopover.svelte";
  import Icon from "@app/components/Icon.svelte";
  import RepoAvatar from "@app/components/RepoAvatar.svelte";
  import ShareButton from "@app/components/ShareButton.svelte";
  import UserAvatar from "@app/components/UserAvatar.svelte";
  import VisibilityBadge from "@app/components/VisibilityBadge.svelte";

  interface Props {
    repo: RepoInfo;
    config: Config;
  }

  const { repo, config }: Props = $props();

  const project = $derived(repo.payloads["xyz.radicle.project"]!);

  // The teams this repository names in its dev.radicle.teams.v1 identity-document
  // payload. Loaded after render (off the navigation path); a repo without the
  // payload simply returns none, so the block is hidden.
  let teams: RepoTeam[] = $state([]);
  let teamsRid: string | undefined;

  $effect(() => {
    const requested = repo.rid;
    if (teamsRid === requested) {
      return;
    }
    teamsRid = requested;
    teams = [];
    void invoke<RepoTeam[]>("repo_teams", { rid: requested })
      .then(result => {
        if (teamsRid === requested) {
          teams = result;
        }
      })
      .catch(() => {
        if (teamsRid === requested) {
          teamsRid = undefined;
        }
      });
  });
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
  .team-avatar {
    width: 1.25rem;
    height: 1.25rem;
    overflow: hidden;
    flex-shrink: 0;
    display: flex;
  }
  /* A team that does not list this repository back is greyed and desaturated —
     the app's existing idiom for something present but not carrying weight. */
  .team-avatar.oneway {
    filter: grayscale(1);
    opacity: 0.5;
  }
  .popover-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  .popover {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    max-width: 26rem;
  }
  .popover-title {
    font: var(--txt-body-m-semibold);
    color: var(--color-text-primary);
  }
  .popover p {
    margin: 0;
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
  }
  .popover :global(code) {
    font-family: var(--font-family-code);
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

{#snippet teamPopover(team: RepoTeam)}
  {@const name = team.name ?? formatRepositoryId(team.rid)}
  <div class="popover">
    <div class="popover-header">
      <span class="team-avatar" class:oneway={!team.mutual}>
        <RepoAvatar
          name={team.name ?? ""}
          rid={team.rid}
          styleWidth="1.25rem" />
      </span>
      <span class="popover-title">{name}</span>
    </div>
    {#if team.mutual}
      <!-- prettier-ignore -->
      <p>This repository names {name} in its identity document, under <code>dev.radicle.teams.v1</code>, and {name} lists this repository in its own <code>.radicle/team.json</code>.</p>
      <p>
        Two public statements that agree. Nothing has been checked, and neither
        grants anything.
      </p>
    {:else}
      <!-- prettier-ignore -->
      <p>This repository names {name} in its identity document, under <code>dev.radicle.teams.v1</code>. {name}'s own file does not list this repository.</p>
      <p>
        Usually the team dropped it and the reference was left behind. The
        reference lives in this repository's identity, so only its delegates can
        remove it, through the CLI.
      </p>
    {/if}
  </div>
{/snippet}

<div class="header">
  <div class="project txt-selectable">
    <div class="name txt-overflow">{project.data.name}</div>
    {#if project.data.description}
      <div class="description txt-overflow">{project.data.description}</div>
    {/if}
  </div>

  <div class="meta">
    <VisibilityBadge type={repo.visibility.type} />

    {#if teams.length > 0}
      <div class="meta-item">
        <span class="meta-label">Teams</span>
        <div class="avatars">
          {#each teams as team (team.rid)}
            <HoverPopover placement="bottom-start" stylePadding="1rem">
              {#snippet toggle()}
                <div class="team-avatar" class:oneway={!team.mutual}>
                  <RepoAvatar
                    name={team.name ?? ""}
                    rid={team.rid}
                    styleWidth="1.25rem" />
                </div>
              {/snippet}
              {#snippet popover()}
                {@render teamPopover(team)}
              {/snippet}
            </HoverPopover>
          {/each}
        </div>
      </div>
    {/if}

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
