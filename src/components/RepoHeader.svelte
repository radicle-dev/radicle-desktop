<script lang="ts">
  import type { RepoInfo } from "@bindings/RepoInfo";

  import Fill from "./Fill.svelte";
  import Icon from "./Icon.svelte";

  export let repo: RepoInfo;
  export let selfDid: string;
  export let emphasizedTitle: boolean = true;

  $: project = repo.payloads["xyz.radicle.project"]!;
</script>

<style>
  .header {
    justify-content: space-between;
    width: 100%;
  }
</style>

<div class="global-flex header txt-small">
  <div class="global-flex">
    <Fill styleWidth="1.5rem" styleHeight="24px" variant="ghost">
      {project.data.name[0]}
    </Fill>
    {#if emphasizedTitle}
      <span class="txt-regular txt-semibold">{project.data.name}</span>
    {:else}
      <span class="txt-small txt-semibold">{project.data.name}</span>
    {/if}
  </div>
  <div class="global-flex">
    {#if repo.visibility.type === "private"}
      <Fill variant="private" styleWidth="24px" styleHeight="24px">
        <div style:color="var(--color-foreground-yellow)">
          <Icon name="lock" />
        </div>
      </Fill>
    {/if}
    {#if repo.delegates.find(x => x.did === selfDid)}
      <Fill variant="delegate" styleWidth="24px" styleHeight="24px">
        <div style:color="var(--color-fill-primary)">
          <Icon name="delegate" />
        </div>
      </Fill>
    {/if}
    <div class="global-flex">
      <Fill variant="ghost" styleHeight="24px" stylePadding="0 4px">
        <Icon name="seedling" />
        <span style:line-height="16px">{repo.seeding}</span>
      </Fill>
    </div>
  </div>
</div>
