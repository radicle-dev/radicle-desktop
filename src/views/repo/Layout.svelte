<script lang="ts">
  import type { RepoInfo } from "@bindings/RepoInfo";

  import Header from "@app/components/Header.svelte";
  import Link from "@app/components/Link.svelte";

  export let repo: RepoInfo;

  $: project = repo.payloads["xyz.radicle.project"]!;
</script>

<Header currentPage="Repositories" />
<div>{project.data.name}</div>
<div>{repo.rid}</div>

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
