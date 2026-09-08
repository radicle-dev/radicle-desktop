<script lang="ts">
  import type { TeamMember } from "@bindings/repo/TeamMember";

  import { invoke } from "@app/lib/invoke";
  import { publicKeyFromDid, truncateDid, truncateId } from "@app/lib/utils";

  import Icon from "@app/components/Icon.svelte";
  import UserAvatar from "@app/components/UserAvatar.svelte";

  interface Props {
    rid: string;
    selfPublicKey: string;
  }

  const { rid, selfPublicKey }: Props = $props();

  let roster: TeamMember[] = $state([]);

  $effect(() => {
    void (async () => {
      try {
        roster = await invoke<TeamMember[]>("team_members", { rid });
      } catch {
        roster = [];
      }
    })();
  });

  const keyRows = $derived(
    roster
      .filter(m => m.kind === "key")
      .map(m => ({ did: m.id, publicKey: publicKeyFromDid(m.id) })),
  );
  const actorRows = $derived(roster.filter(m => m.kind === "actor"));

  const rows = $derived(keyRows);

  // eslint-disable-next-line svelte/prefer-svelte-reactivity -- must stay non-reactive: a reactive Set would re-trigger the effect on every add and reintroduce the request loop
  const requested = new Set<string>();
  let aliases: Record<string, string | null> = $state({});

  $effect(() => {
    for (const { publicKey } of rows) {
      if (requested.has(publicKey)) {
        continue;
      }
      requested.add(publicKey);
      void (async () => {
        try {
          const alias = await invoke<string | null>("alias", {
            nid: publicKey,
          });
          aliases = { ...aliases, [publicKey]: alias };
        } catch {
          aliases = { ...aliases, [publicKey]: null };
        }
      })();
    }
  });
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
    min-width: 0;
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
  .primary {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    min-width: 0;
  }
  .alias {
    font: var(--txt-body-m-medium);
    color: var(--color-text-primary);
  }
  .nid {
    font: var(--txt-code-small);
    color: var(--color-text-primary);
  }
  .did {
    font: var(--txt-code-small);
    color: var(--color-text-secondary);
  }
  .badge {
    display: inline-flex;
    align-items: center;
    padding: 0 0.25rem;
    border: 1px solid var(--color-border-mid);
    border-radius: var(--border-radius-sm);
    font: var(--txt-body-s-regular);
    color: var(--color-text-secondary);
  }
  .verified {
    display: inline-flex;
    color: var(--color-text-brand);
  }
  .status-label {
    margin-left: auto;
    font: var(--txt-body-s-regular);
    color: var(--color-text-secondary);
    white-space: nowrap;
    flex-shrink: 0;
  }
  .empty {
    padding: 0.75rem 1rem;
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
  }
</style>

{#if actorRows.length > 0}
  <div class="list">
    {#each actorRows as member (member.id)}
      <div class="row">
        <span class="avatar"><Icon name="repository" /></span>
        <div class="body">
          <span class="primary txt-overflow">
            {#if member.name}
              <span class="alias txt-overflow">{member.name}</span>
            {:else}
              <span class="nid">
                {truncateId(member.id.replace("rad:", ""))}
              </span>
            {/if}
          </span>
          <span class="did">actor repository</span>
        </div>
        {#if member.attestation === "attested"}
          <span class="status-label">attested</span>
        {:else if member.attestation === "unconfirmed"}
          <span class="status-label">not asserted back</span>
        {:else if member.attestation === "unknown"}
          <span class="status-label">not replicated</span>
        {/if}
      </div>
    {/each}
  </div>
{/if}

{#if rows.length > 0}
  <div class="list">
    {#each rows as { did, publicKey } (did)}
      {@const resolved = publicKey in aliases}
      {@const alias = resolved ? aliases[publicKey] : undefined}
      {@const isYou = publicKey === selfPublicKey}
      <div class="row">
        <span class="avatar">
          <UserAvatar nodeId={publicKey} styleWidth="2.5rem" />
        </span>
        <div class="body">
          <span class="primary txt-overflow">
            {#if alias}
              <span class="alias txt-overflow">{alias}</span>
            {:else}
              <span class="nid">{truncateId(publicKey)}</span>
            {/if}
            {#if isYou}
              <span class="badge">you</span>
              <span class="verified"><Icon name="badge" /></span>
            {/if}
          </span>
          <span class="did">{truncateDid(did)}</span>
        </div>
        {#if resolved && alias === null && !isYou}
          <span class="status-label">not known to your node</span>
        {/if}
      </div>
    {/each}
  </div>
{:else if actorRows.length === 0}
  <div class="empty">No members.</div>
{/if}
