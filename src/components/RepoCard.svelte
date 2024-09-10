<script lang="ts">
  import type { RepoInfo } from "@bindings/RepoInfo";

  import { formatRepositoryId, formatTimestamp } from "@app/lib/utils";

  import Border from "./Border.svelte";
  import Fill from "./Fill.svelte";
  import Icon from "./Icon.svelte";

  export let repo: RepoInfo;
  export let selfDid: string;
  export let onclick: (() => void) | undefined = undefined;

  $: project = repo.payloads["xyz.radicle.project"]!;
</script>

<style>
  .header {
    justify-content: space-between;
  }
  .footer {
    margin-top: 1rem;
    justify-content: space-between;
  }
  .title {
    display: flex;
    color: var(--color-fill-gray);
    margin-top: 4px;
  }
  .container {
    width: 100%;
  }
</style>

<Border
  variant="ghost"
  styleWidth="100%"
  stylePadding="8px 12px"
  hoverable
  {onclick}>
  <div class="container txt-small">
    <div class="global-flex header">
      <div class="global-flex">
        <Fill styleWidth="1.5rem" styleHeight="24px" variant="ghost">
          {project.data.name[0]}
        </Fill>{project.data.name}
      </div>
      <div class="global-flex">
        {#if repo.visibility.type === "private"}
          <Fill variant="private" styleWidth="24px" styleHeight="24px">
            <div style:color="var(--color-foreground-yellow)">
              <Icon name="lock" />
            </div>
          </Fill>
        {/if}
        {#if repo.delegates.find(x => x.id === selfDid)}
          <Fill variant="delegate" styleWidth="24px" styleHeight="24px">
            <div style:color="var(--color-fill-primary)">
              <Icon name="delegate" />
            </div>
          </Fill>
        {/if}
        <div class="global-flex">
          <Fill variant="ghost" styleHeight="24px" stylePadding="0 4px">
            <Icon name="seedling" />
            {repo.seeding}
          </Fill>
        </div>
      </div>
    </div>

    <div class="title">
      {#if project.data.description}
        {project.data.description}
      {:else}
        No description
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
