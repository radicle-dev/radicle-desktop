<script lang="ts">
  import { invoke } from "@app/lib/invoke";
  import * as router from "@app/lib/router";
  import type { SidebarData } from "@app/lib/router/definitions";
  import { formatRepositoryId } from "@app/lib/utils";

  import Button from "@app/components/Button.svelte";
  import HoverPopover from "@app/components/HoverPopover.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Label from "@app/components/Label.svelte";
  import RepoAvatar from "@app/components/RepoAvatar.svelte";

  interface Props {
    repos: string[];
    assertingRids: string[];
    sidebarData: SidebarData;
  }

  const { repos, assertingRids, sidebarData }: Props = $props();

  const rows = $derived(
    repos.map(rid => ({
      rid,
      summary: sidebarData.repos.find(repo => repo.rid === rid),
    })),
  );

  // `seeding` is in-flight; `seeded` marks a repo whose seeding policy we have
  // set. The node then fetches it in the background, so the row becomes a
  // normal seeded repo only once replication completes and this view is
  // reloaded — until then we show a terminal "Seeding…" label rather than a
  // stuck, permanently-disabled button.
  let seeding: Record<string, boolean> = $state({});
  let seeded: Record<string, boolean> = $state({});

  async function seed(rid: string) {
    seeding = { ...seeding, [rid]: true };
    try {
      await invoke<null>("seed", { rid });
      seeded = { ...seeded, [rid]: true };
    } catch (error) {
      console.error("Seeding failed", String(error));
    } finally {
      seeding = { ...seeding, [rid]: false };
    }
  }
</script>

<style>
  .list {
    display: flex;
    flex-direction: column;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--color-border-subtle);
    text-decoration: none;
    color: var(--color-text-primary);
    min-width: 0;
  }
  a.row:hover {
    background-color: var(--color-surface-subtle);
  }
  .avatar {
    width: 2.5rem;
    height: 2.5rem;
    flex-shrink: 0;
    overflow: hidden;
    border-radius: var(--border-radius-sm);
    display: flex;
  }
  .body {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
    min-width: 0;
  }
  .name-line {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
  }
  .name {
    font: var(--txt-body-m-medium);
    color: var(--color-text-primary);
  }
  .meta {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
    min-width: 0;
    color: var(--color-text-secondary);
  }
  .rid {
    font: var(--txt-code-small);
    color: var(--color-text-secondary);
    white-space: nowrap;
  }
  .description {
    font: var(--txt-body-s-regular);
    color: var(--color-text-secondary);
  }
  .status {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex-shrink: 0;
  }
  .status-label {
    font: var(--txt-body-s-regular);
    color: var(--color-text-secondary);
    white-space: nowrap;
  }
  .empty {
    padding: 0.75rem 1rem;
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
  }
  .popover {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    max-width: 26rem;
  }
  .popover-title {
    font: var(--txt-body-m-semibold);
    color: var(--color-text-primary);
  }
  .popover p {
    margin: 0;
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
  }
  .popover :global(code) {
    font-family: var(--font-family-code);
  }
</style>

{#snippet attested()}
  <HoverPopover placement="bottom-start" stylePadding="1rem">
    {#snippet toggle()}
      <Label label="attested" icon="checkmark" />
    {/snippet}
    {#snippet popover()}
      <div class="popover">
        <div class="popover-title">Both sides name each other</div>
        <!-- prettier-ignore -->
        <p>This team lists the repository in <code>.radicle/team.json</code>, and the repository names this team in its identity document, under <code>dev.radicle.teams.v1</code>.</p>
        <p>
          Neither file grants anything and nothing has been checked. They are
          public statements, trusted the way a README is trusted.
        </p>
        <p>
          Most repositories never publish the second file. Its absence says
          nothing either way.
        </p>
      </div>
    {/snippet}
  </HoverPopover>
{/snippet}

{#if rows.length > 0}
  <div class="list">
    {#each rows as { rid, summary } (rid)}
      {#if summary}
        <a
          class="row"
          href={router.routeToPath({ resource: "repo.home", rid })}>
          <span class="avatar">
            <RepoAvatar name={summary.name} {rid} styleWidth="2.5rem" />
          </span>
          <div class="body">
            <div class="name-line">
              <span class="name txt-overflow">{summary.name}</span>
              {#if assertingRids.includes(rid)}
                {@render attested()}
              {/if}
            </div>
            <span class="meta txt-overflow">
              <span class="rid">{formatRepositoryId(rid)}</span>
              {#if summary.description}
                <span class="description txt-overflow">
                  {summary.description}
                </span>
              {/if}
            </span>
          </div>
        </a>
      {:else}
        <div class="row">
          <span class="avatar">
            <RepoAvatar name="" {rid} styleWidth="2.5rem" />
          </span>
          <div class="body">
            <span class="rid">{formatRepositoryId(rid)}</span>
          </div>
          <div class="status">
            {#if seeded[rid]}
              <span class="status-label">Seeding…</span>
            {:else}
              <span class="status-label">not seeded locally</span>
              <Button
                variant="outline"
                disabled={seeding[rid]}
                onclick={() => seed(rid)}>
                <Icon name="seed" />
                {seeding[rid] ? "Seeding…" : "Seed"}
              </Button>
            {/if}
          </div>
        </div>
      {/if}
    {/each}
  </div>
{:else}
  <div class="empty">No repositories.</div>
{/if}
