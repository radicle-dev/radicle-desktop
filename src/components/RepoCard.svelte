<script lang="ts">
  import type { RepoInfo } from "@bindings/RepoInfo";

  import { formatRepositoryId, formatTimestamp } from "@app/lib/utils";

  import Border from "./Border.svelte";
  import Icon from "./Icon.svelte";
  import RepoHeader from "./RepoHeader.svelte";

  export let repo: RepoInfo;
  export let selfDid: string;
  export let onclick: (() => void) | undefined = undefined;

  $: project = repo.payloads["xyz.radicle.project"]!;
</script>

<style>
  .footer {
    margin-top: 1rem;
    justify-content: space-between;
  }
  .title {
    color: var(--color-fill-gray);
    margin-top: 4px;
    overflow: hidden;
    text-overflow: ellipsis;
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
  hoverable
  {onclick}>
  <div class="container txt-small">
    <RepoHeader {repo} {selfDid} />

    <div class="title" title={project.data.description}>
      {#if project.data.description}
        {project.data.description}
      {:else}
        No description.
      {/if}
    </div>
    <div class="global-oid">{formatRepositoryId(repo.rid)}</div>

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
        Updated {formatTimestamp(project.meta.lastCommit)}
      </span>
    </div>
  </div>
</Border>
