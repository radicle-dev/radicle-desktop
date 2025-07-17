<script lang="ts">
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import * as router from "@app/lib/router";
  import { formatRepositoryId, formatTimestamp } from "@app/lib/utils";

  import Border from "@app/components/Border.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import RepoHeader from "@app/components/RepoHeader.svelte";

  interface Props {
    repo: RepoInfo;
    selfDid: string;
    focussed?: boolean;
  }

  const { repo, selfDid, focussed }: Props = $props();

  const project = $derived(repo.payloads["xyz.radicle.project"]!);
</script>

<style>
  .footer {
    margin-top: 1rem;
    justify-content: space-between;
  }
  .description {
    color: var(--color-fill-gray);
    margin-top: 0.25rem;
  }
  .content {
    width: 100%;
    padding: 0.5rem 0.75rem;
    cursor: pointer;
  }
</style>

<Border
  variant={focussed ? "secondary" : "ghost"}
  styleWidth="100%"
  styleOverflow="hidden"
  hoverable>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="content txt-small"
    onclick={event => {
      if (!event.defaultPrevented)
        void router.push({
          resource: "repo.home",
          rid: repo.rid,
        });
    }}>
    <RepoHeader {repo} {selfDid} />

    <div class="description txt-overflow" title={project.data.description}>
      {#if project.data.description}
        {project.data.description}
      {:else}
        No description.
      {/if}
    </div>
    <Id
      ariaLabel="repo-id"
      clipboard={repo.rid}
      shorten={false}
      variant="oid"
      id={formatRepositoryId(repo.rid)} />

    <div class="global-flex footer">
      <div class="global-flex">
        <div class="global-flex" style:gap="0.25rem">
          <Icon name="issue" />{project.meta.issues.open}
        </div>
        <div class="global-flex" style:gap="0.25rem">
          <Icon name="patch" />{project.meta.patches.open}
        </div>
      </div>
      <span style:color="var(--color-fill-gray)">
        Updated {formatTimestamp(repo.lastCommitTimestamp)}
      </span>
    </div>
  </div>
</Border>
