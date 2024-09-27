<script lang="ts">
  import type { Patch } from "@bindings/Patch";
  import type { Stats } from "@bindings/Stats";

  import {
    formatOid,
    formatTimestamp,
    patchStatusBackgroundColor,
    patchStatusColor,
  } from "@app/lib/utils";
  import { invoke } from "@tauri-apps/api/core";
  import { push } from "@app/lib/router";

  import DiffStatBadge from "./DiffStatBadge.svelte";
  import Icon from "./Icon.svelte";
  import InlineTitle from "./InlineTitle.svelte";
  import NodeId from "./NodeId.svelte";

  export let patch: Patch;
  export let rid: string;
</script>

<style>
  .patch-teaser {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.25rem;
    height: 5rem;
    background-color: var(--color-background-float);
    padding: 1rem;
    cursor: pointer;
    font-size: var(--font-size-regular);
  }
  .patch-teaser:hover {
    background-color: var(--color-fill-float-hover);
  }
  .status {
    padding: 0;
    margin-right: 1rem;
  }
  .patch-teaser:first-of-type {
    clip-path: var(--3px-top-corner-fill);
  }
  .patch-teaser:last-of-type {
    clip-path: var(--3px-bottom-corner-fill);
  }
  .patch-teaser:only-of-type {
    clip-path: var(--3px-corner-fill);
  }
</style>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  tabindex="0"
  role="button"
  class="patch-teaser"
  onclick={() => {
    void push({ resource: "repo.patch", rid, patch: patch.id });
  }}>
  <div class="global-flex">
    <div
      class="global-counter status"
      style:color={patchStatusColor[patch.state.status]}
      style:background-color={patchStatusBackgroundColor[patch.state.status]}>
      <Icon name="patch" />
    </div>
    <div
      class="global-flex"
      style:flex-direction="column"
      style:align-items="flex-start">
      <InlineTitle content={patch.title} />
      <div class="global-flex txt-small">
        <NodeId
          nodeId={patch.author.did.replace("did:key:", "")}
          alias={patch.author.alias} />
        opened
        <div class="global-oid">{formatOid(patch.id)}</div>
        {formatTimestamp(patch.timestamp)}
      </div>
    </div>
  </div>
  <div class="global-flex">
    {#await invoke<Stats>( "diff_stats", { rid, base: patch.base, head: patch.head }, ) then stats}
      <DiffStatBadge {stats} />
    {/await}
    {#each patch.labels as label}
      <div class="global-counter txt-small">{label}</div>
    {/each}

    <div class="txt-small global-flex" style:gap="0.25rem">
      <Icon name="revision" />
      {patch.revisionCount}
    </div>
  </div>
</div>
