<script lang="ts">
  import type { Patch } from "@bindings/cob/patch/Patch";
  import type { PatchStatus } from "@app/views/repo/router";
  import type { Stats } from "@bindings/cob/Stats";

  import {
    authorForNodeId,
    formatTimestamp,
    patchStatusBackgroundColor,
    patchStatusColor,
  } from "@app/lib/utils";
  import { invoke } from "@app/lib/invoke";
  import { push } from "@app/lib/router";

  import DiffStatBadge from "@app/components/DiffStatBadge.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import InlineTitle from "@app/components/InlineTitle.svelte";
  import Label from "@app/components/Label.svelte";
  import NodeId from "@app/components/NodeId.svelte";

  interface Props {
    compact?: boolean;
    loadPatch?: (rid: string, patchId: string) => void;
    patch: Patch;
    rid: string;
    selected?: boolean;
    status: PatchStatus | undefined;
  }

  const {
    compact = false,
    loadPatch,
    patch,
    rid,
    selected = false,
    status,
  }: Props = $props();
</script>

<style>
  .patch-teaser {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.25rem;
    min-height: 5rem;
    background-color: var(--color-background-float);
    padding: 1rem;
    cursor: pointer;
    font-size: var(--font-size-regular);
    word-break: break-word;
  }
  .selected {
    background-color: var(--color-fill-float-hover);
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
  class:selected
  class="patch-teaser"
  style:align-items="flex-start"
  onclick={() => {
    if (loadPatch) {
      loadPatch(rid, patch.id);
    } else {
      void push({
        resource: "repo.patch",
        rid,
        patch: patch.id,
        status,
        reviewId: undefined,
      });
    }
  }}>
  <div class="global-flex" style:align-items="flex-start">
    <div
      class="global-counter status"
      style:color={patchStatusColor[patch.state.status]}
      style:background-color={patchStatusBackgroundColor[patch.state.status]}>
      <Icon
        name={patch.state.status === "open"
          ? "patch"
          : `patch-${patch.state.status}`} />
    </div>
    <div
      class="global-flex"
      style:flex-direction="column"
      style:align-items="flex-start">
      <InlineTitle content={patch.title} />
      <div class="global-flex txt-small" style:flex-wrap="wrap">
        <NodeId {...authorForNodeId(patch.author)} />
        opened
        <Id id={patch.id} variant="oid" />
        {formatTimestamp(patch.timestamp)}
      </div>
    </div>
  </div>

  <div class="global-flex">
    {#if !compact}
      {#await invoke<Stats>( "diff_stats", { rid, base: patch.base, head: patch.head }, ) then stats}
        <DiffStatBadge {stats} />
      {/await}

      {#each patch.labels as label}
        <Label {label} />
      {/each}
    {/if}
    <div
      class="txt-small global-flex"
      style:gap="0.25rem"
      style:white-space="nowrap">
      <Icon name="revision" />
      {patch.revisionCount}
    </div>
  </div>
</div>
