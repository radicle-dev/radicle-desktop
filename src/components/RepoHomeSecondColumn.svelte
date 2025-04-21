<script lang="ts">
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import Border from "./Border.svelte";
  import Icon from "./Icon.svelte";
  import Link from "./Link.svelte";
  import RepoTeaser from "./RepoTeaser.svelte";
  import Settings from "./Settings.svelte";

  interface Props {
    repo: RepoInfo;
  }

  const { repo }: Props = $props();

  const project = $derived(repo.payloads["xyz.radicle.project"]!);
</script>

<style>
  .container {
    display: flex;
    flex-direction: column;
    height: 100%;
    justify-content: space-between;
  }
  .tab {
    align-items: center;
    background-color: var(--color-background-float);
    clip-path: var(--1px-corner-fill);
    display: flex;
    font-size: var(--font-size-small);
    justify-content: space-between;
    padding: 0.5rem 0.25rem 0.5rem 0.5rem;
    width: 100%;
  }
  .tab:not(.active) {
    color: var(--color-foreground-dim);
  }
  .tab:not(.active):hover {
    background-color: var(--color-fill-float-hover);
  }
  .active {
    background-color: var(--color-background-default);
    font-weight: var(--font-weight-semibold);
    padding: 0.25rem 0.25rem 0.25rem 0.5rem;
  }
</style>

<div class="container">
  <div>
    <div style:margin-bottom="0.75rem">
      <Border
        variant="ghost"
        styleBackgroundColor="var(--color-background-default)">
        <div class="tab active" style:color="var(--color-foreground-contrast)">
          <RepoTeaser name={project.data.name} seeding={repo.seeding} />
        </div>
      </Border>
    </div>

    <div style:margin-bottom="0.5rem">
      <Link
        styleWidth="100%"
        underline={false}
        route={{ resource: "repo.issues", rid: repo.rid, status: "open" }}>
        <div
          class="tab"
          style:color="var(--color-foreground-contrast)"
          style:padding-left="0.75rem">
          <div class="global-flex"><Icon name="issue" />Issues</div>
          <div class="global-counter">
            {project.meta.issues.open + project.meta.issues.closed}
          </div>
        </div>
      </Link>
    </div>

    <div style:margin-top="0.5rem">
      <Link
        styleWidth="100%"
        underline={false}
        route={{ resource: "repo.patches", rid: repo.rid, status: "open" }}>
        <div
          class="tab"
          style:color="var(--color-foreground-contrast)"
          style:padding-left="0.75rem">
          <div class="global-flex"><Icon name="patch" />Patches</div>
          <div class="global-counter">
            {project.meta.patches.draft +
              project.meta.patches.open +
              project.meta.patches.archived +
              project.meta.patches.merged}
          </div>
        </div>
      </Link>
    </div>
  </div>

  <Settings
    compact={false}
    popoverProps={{
      popoverPositionBottom: "3rem",
      popoverPositionLeft: "0",
    }} />
</div>
