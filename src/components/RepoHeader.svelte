<script lang="ts">
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import * as router from "@app/lib/router";

  import Icon from "@app/components/Icon.svelte";

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
    width: 100%;
    gap: 0.5rem;
  }
</style>

<div class="header txt-small">
  <div class="global-counter" style:background-color="var(--color-fill-ghost)">
    {project.data.name[0]}
  </div>
  <div
    role="button"
    style:min-width="0"
    style:margin-right="auto"
    tabindex="0"
    onclick={event => {
      event.preventDefault();
      void router.push({
        resource: "repo.home",
        rid: repo.rid,
      });
    }}
    onkeypress={event => {
      if (event.key === "Enter" || event.key === " ") {
        event.preventDefault();
        void router.push({
          resource: "repo.home",
          rid: repo.rid,
        });
      }
    }}
    title={project.data.name}>
    <div class="txt-regular txt-overflow txt-semibold" style:max-width="100%">
      {project.data.name}
    </div>
  </div>
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
  <div
    class="global-counter"
    style:padding="0 0.375rem"
    style:background-color="var(--color-fill-ghost)"
    style:gap="0.25rem">
    <Icon name="seedling-filled" />
    {repo.seeding}
  </div>
</div>
