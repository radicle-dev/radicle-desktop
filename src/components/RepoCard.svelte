<script lang="ts">
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import { formatRepositoryId, formatTimestamp } from "@app/lib/utils";

  import Border from "./Border.svelte";
  import Icon from "./Icon.svelte";
  import RepoHeader from "./RepoHeader.svelte";
  import Id from "./Id.svelte";

  interface Props {
    repo: RepoInfo;
    selfDid: string;
    onclick?: () => void;
  }

  const { repo, selfDid, onclick }: Props = $props();

  const project = $derived(repo.payloads["xyz.radicle.project"]!);
</script>

<style>
  .footer {
    margin-top: 1rem;
    justify-content: space-between;
  }
  .description {
    color: var(--color-fill-gray);
    margin-top: 4px;
  }
  .container {
    width: 100%;
  }
</style>

<Border
  variant="ghost"
  styleCursor="pointer"
  styleWidth="100%"
  stylePadding="8px 12px"
  styleOverflow="hidden"
  hoverable
  {onclick}>
  <div class="container txt-small">
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
        <div class="global-flex" style:gap="4px">
          <Icon name="issue" />{project.meta.issues.open}
        </div>
        <div class="global-flex" style:gap="4px">
          <Icon name="patch" />{project.meta.patches.open}
        </div>
      </div>
      <span style:color="var(--color-fill-gray)">
        Updated {formatTimestamp(repo.lastCommitTimestamp)}
      </span>
    </div>
  </div>
</Border>
