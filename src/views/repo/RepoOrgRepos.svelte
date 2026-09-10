<script lang="ts">
  import type { Commit } from "@bindings/repo/Commit";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import type { OrgParseResult } from "@app/lib/org";
  import type { SidebarData } from "@app/lib/router/definitions";

  import OrgRepoList from "@app/components/OrgRepoList.svelte";
  import RepoHeader from "@app/components/RepoHeader.svelte";
  import ScrollArea from "@app/components/ScrollArea.svelte";
  import SourceHeader from "@app/components/SourceHeader.svelte";

  import Layout from "./Layout.svelte";

  interface Props {
    repo: RepoInfo;
    oid: string;
    commit: Commit;
    peer?: string;
    revision?: string;
    org: OrgParseResult;
    assertingRids: string[];
    sidebarData: SidebarData;
  }

  const {
    repo,
    oid,
    commit,
    peer,
    revision,
    org,
    assertingRids,
    sidebarData,
  }: Props = $props();

  const baseRoute = $derived({
    resource: "repo.home" as const,
    rid: repo.rid,
    peer,
    revision,
  });
</script>

<style>
  .page {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }
  .degraded {
    padding: 0.75rem 1rem;
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
  }
</style>

<Layout selfScroll>
  <div class="page">
    <RepoHeader {repo} config={sidebarData.config} />
    <SourceHeader
      {repo}
      {peer}
      {revision}
      {oid}
      {commit}
      {baseRoute}
      isOrg
      active="repos" />
    <ScrollArea style="flex: 1; min-height: 0;">
      <div>
        {#if org.status === "ok"}
          <OrgRepoList repos={org.org.repos} {assertingRids} {sidebarData} />
        {:else if org.status === "unsupported-version"}
          <div class="degraded">
            This org file uses version {org.version}, which this app doesn't
            understand.
          </div>
        {:else}
          <div class="degraded">{org.message}</div>
        {/if}
      </div>
    </ScrollArea>
  </div>
</Layout>
