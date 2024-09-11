<script lang="ts">
  import type { RepoInfo } from "@bindings/RepoInfo";

  import CopyableId from "@app/components/CopyableId.svelte";
  import Header from "@app/components/Header.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Link from "@app/components/Link.svelte";

  export let repo: RepoInfo;

  $: project = repo.payloads["xyz.radicle.project"]!;
</script>

<style>
  .header {
    position: sticky;
    top: 0;
  }
</style>

<div style:height="fit-content">
  <div class="header">
    <Header>
      <svelte:fragment slot="icon-left">
        <Icon name="sidebar" />
      </svelte:fragment>
      <svelte:fragment slot="center">
        <CopyableId id={repo.rid} />
      </svelte:fragment>
    </Header>
  </div>
  <div>{project.data.name}</div>

  Issues
  <Link route={{ resource: "repo.issues", rid: repo.rid, status: "open" }}>
    Open
  </Link>
  <Link route={{ resource: "repo.issues", rid: repo.rid, status: "closed" }}>
    Closed
  </Link>

  <br />
  Patches
  <Link route={{ resource: "repo.patches", rid: repo.rid, status: "draft" }}>
    Draft
  </Link>
  <Link route={{ resource: "repo.patches", rid: repo.rid, status: "open" }}>
    Open
  </Link>
  <Link route={{ resource: "repo.patches", rid: repo.rid, status: "archived" }}>
    Archived
  </Link>
  <Link route={{ resource: "repo.patches", rid: repo.rid, status: "merged" }}>
    Merged
  </Link>
  <slot />
</div>
