<script lang="ts">
  import type { IssueStatus } from "@app/views/repo/router";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import Border from "./Border.svelte";
  import Icon from "./Icon.svelte";
  import Link from "./Link.svelte";
  import RepoTeaser from "./RepoTeaser.svelte";
  import Settings from "./Settings.svelte";

  interface Props {
    status: IssueStatus;
    repo: RepoInfo;
  }

  const { status, repo }: Props = $props();

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
    padding: 8px 4px 8px 8px;
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
  }
  .highlight {
    color: var(--color-foreground-contrast);
  }
  .closed {
    color: var(--color-foreground-red);
  }
  .open {
    color: var(--color-foreground-success);
  }
</style>

<div class="container">
  <div>
    <div style:margin-bottom="0.5rem" style:padding-left="0.75rem">
      <RepoTeaser name={project.data.name} seeding={repo.seeding} />
    </div>

    <Border
      variant="ghost"
      styleFlexDirection="column"
      styleGap="2px"
      styleBackgroundColor="var(--color-background-float)">
      <Link
        styleWidth="100%"
        underline={false}
        route={{ resource: "repo.issues", rid: repo.rid, status: "all" }}>
        <div class="tab active">
          <div class="global-flex"><Icon name="issue" />Issues</div>
          <div class="global-counter">
            {project.meta.issues.open + project.meta.issues.closed}
          </div>
        </div>
      </Link>

      <Link
        styleWidth="100%"
        underline={false}
        route={{
          resource: "repo.issues",
          rid: repo.rid,
          status: "open",
        }}>
        <div class="tab" class:active={status === "open"}>
          <div
            class="global-flex"
            class:open={["open", "all"].includes(status)}>
            <Icon name="issue" />Open
          </div>
          <div class="global-counter" class:highlight={status === "all"}>
            {project.meta.issues.open}
          </div>
        </div>
      </Link>

      <Link
        styleWidth="100%"
        underline={false}
        route={{
          resource: "repo.issues",
          rid: repo.rid,
          status: "closed",
        }}>
        <div class="tab" class:active={status === "closed"}>
          <div
            class="global-flex"
            class:closed={["closed", "all"].includes(status)}>
            <Icon name="issue-closed" />Closed
          </div>
          <div class="global-counter" class:highlight={status === "all"}>
            {project.meta.issues.closed}
          </div>
        </div>
      </Link>
    </Border>

    <div style:margin-top="0.5rem">
      <Link
        styleWidth="100%"
        underline={false}
        route={{ resource: "repo.patches", rid: repo.rid, status: "open" }}>
        <div
          class="tab"
          style:color="var(--color-foreground-contrast)"
          style:padding-left="12px">
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
