<script lang="ts">
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import Icon from "./Icon.svelte";

  interface Props {
    repo: RepoInfo;
    selfDid: string;
  }

  const { repo, selfDid }: Props = $props();

  const project = $derived(repo.payloads["xyz.radicle.project"]!);
</script>

<style>
  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    gap: 0.5rem;
  }
</style>

<div class="header txt-small">
  <div class="global-flex txt-overflow">
    <div
      class="global-counter"
      style:background-color="var(--color-fill-ghost)">
      {project.data.name[0]}
    </div>
    <span
      title={project.data.name}
      class="txt-regular txt-overflow txt-semibold">
      {project.data.name}
    </span>
  </div>
  <div class="global-flex">
    {#if repo.visibility.type === "private"}
      <div
        class="global-counter"
        style:padding="0"
        style:background-color="var(--color-fill-private)">
        <div style:color="var(--color-foreground-yellow)">
          <Icon name="lock" />
        </div>
      </div>
    {/if}
    {#if repo.delegates.find(x => x.did === selfDid)}
      <div
        class="global-counter"
        style:padding="0"
        style:background-color="var(--color-fill-delegate)">
        <div style:color="var(--color-fill-primary)">
          <Icon name="delegate" />
        </div>
      </div>
    {/if}
    <div class="global-flex">
      <div
        class="global-counter"
        style:padding="0 0.375rem"
        style:background-color="var(--color-fill-ghost)"
        style:gap="0.25rem">
        <Icon name="seedling" />
        {repo.seeding}
      </div>
    </div>
  </div>
</div>
