<script lang="ts">
  import type { Commit } from "@bindings/repo/Commit";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import type { SidebarData } from "@app/lib/router/definitions";
  import type { TeamParseResult } from "@app/lib/team";

  import RepoHeader from "@app/components/RepoHeader.svelte";
  import ScrollArea from "@app/components/ScrollArea.svelte";
  import SourceHeader from "@app/components/SourceHeader.svelte";
  import TeamMemberList from "@app/components/TeamMemberList.svelte";

  import Layout from "./Layout.svelte";

  interface Props {
    repo: RepoInfo;
    oid: string;
    commit: Commit;
    peer?: string;
    revision?: string;
    team: TeamParseResult;
    sidebarData: SidebarData;
  }

  const { repo, oid, commit, peer, revision, team, sidebarData }: Props =
    $props();

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
      isTeam
      active="members" />
    <ScrollArea style="flex: 1; min-height: 0;">
      <div>
        {#if team.status === "ok"}
          <TeamMemberList
            members={team.team.members}
            selfPublicKey={sidebarData.config.publicKey} />
        {:else if team.status === "unsupported-version"}
          <div class="degraded">
            This team file uses version {team.version}, which this app doesn't
            understand.
          </div>
        {:else}
          <div class="degraded">{team.message}</div>
        {/if}
      </div>
    </ScrollArea>
  </div>
</Layout>
