<script lang="ts">
  import type { PatchStatus } from "@app/views/repo/router";
  import type { ProjectPayload } from "@bindings/repo/ProjectPayload";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import Border from "./Border.svelte";
  import Icon from "./Icon.svelte";
  import Link from "./Link.svelte";
  import RepoTeaser from "./RepoTeaser.svelte";
  import Settings from "./Settings.svelte";

  interface Props {
    project: ProjectPayload;
    status: PatchStatus | undefined;
    repo: RepoInfo;
  }
  const { project, status, repo }: Props = $props();
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
  .draft {
    color: var(--color-fill-gray);
  }
  .open {
    color: var(--color-fill-success);
  }
  .archived {
    color: var(--color-fill-yellow-iconic);
  }
  .merged {
    color: var(--color-foreground-primary);
  }
</style>

<div class="container">
  <div>
    <div style:margin-bottom="0.5rem" style:padding-left="0.75rem">
      <RepoTeaser name={project.data.name} seeding={repo.seeding} />
    </div>

    <div style:margin-bottom="0.5rem">
      <Link
        styleWidth="100%"
        underline={false}
        route={{ resource: "repo.issues", rid: repo.rid, status: "open" }}>
        <div
          class="tab"
          style:color="var(--color-foreground-contrast)"
          style:padding-left="12px">
          <div class="global-flex"><Icon name="issue" />Issues</div>
          <div class="global-counter">
            {project.meta.issues.open + project.meta.issues.closed}
          </div>
        </div>
      </Link>
    </div>

    <Border
      variant="ghost"
      styleFlexDirection="column"
      styleGap="2px"
      styleBackgroundColor="var(--color-background-float)">
      <Link
        styleWidth="100%"
        underline={false}
        route={{ resource: "repo.patches", rid: repo.rid, status: undefined }}>
        <div class="tab active">
          <div class="global-flex"><Icon name="patch" />Patches</div>
          <div class="global-counter">
            {project.meta.patches.draft +
              project.meta.patches.open +
              project.meta.patches.archived +
              project.meta.patches.merged}
          </div>
        </div>
      </Link>

      <Link
        styleWidth="100%"
        underline={false}
        route={{
          resource: "repo.patches",
          rid: repo.rid,
          status: "open",
        }}>
        <div class="tab" class:active={status === "open"}>
          <div
            class="global-flex"
            class:open={["open", undefined].includes(status)}>
            <Icon name="patch" />
            Open
          </div>
          <div class="global-counter" class:highlight={status === undefined}>
            {project.meta.patches.open}
          </div>
        </div>
      </Link>

      <Link
        styleWidth="100%"
        underline={false}
        route={{
          resource: "repo.patches",
          rid: repo.rid,
          status: "merged",
        }}>
        <div class="tab" class:active={status === "merged"}>
          <div
            class="global-flex"
            class:merged={["merged", undefined].includes(status)}>
            <Icon name="patch" />Merged
          </div>
          <div class="global-counter" class:highlight={status === undefined}>
            {project.meta.patches.merged}
          </div>
        </div>
      </Link>

      <Link
        styleWidth="100%"
        underline={false}
        route={{
          resource: "repo.patches",
          rid: repo.rid,
          status: "archived",
        }}>
        <div class="tab" class:active={status === "archived"}>
          <div
            class="global-flex"
            class:archived={["archived", undefined].includes(status)}>
            <Icon name="patch" />Archived
          </div>
          <div class="global-counter" class:highlight={status === undefined}>
            {project.meta.patches.archived}
          </div>
        </div>
      </Link>
      <Link
        styleWidth="100%"
        underline={false}
        route={{
          resource: "repo.patches",
          rid: repo.rid,
          status: "draft",
        }}>
        <div class="tab" class:active={status === "draft"}>
          <div
            class="global-flex"
            class:draft={["draft", undefined].includes(status)}>
            <Icon name="patch" />
            Draft
          </div>
          <div class="global-counter" class:highlight={status === undefined}>
            {project.meta.patches.draft}
          </div>
        </div>
      </Link>
    </Border>
  </div>

  <Settings
    compact={false}
    popoverProps={{
      popoverPositionBottom: "3rem",
      popoverPositionLeft: "0",
    }} />
</div>
