<script lang="ts">
  import type { Commit } from "@bindings/repo/Commit";
  import type { Diff } from "@bindings/diff/Diff";
  import type { CodeComments } from "./Diff.svelte";
  import type { Revision } from "@bindings/cob/patch/Revision";

  import { invoke } from "@app/lib/invoke";

  import Changeset from "@app/components/Changeset.svelte";
  import CobCommitTeaser from "./CobCommitTeaser.svelte";
  import CommitsContainer from "@app/components/CommitsContainer.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "./Id.svelte";
  import NakedButton from "@app/components/NakedButton.svelte";

  interface Props {
    patchId: string;
    revision: Revision;
    rid: string;
    codeComments?: CodeComments;
  }

  const { patchId, revision, rid, codeComments }: Props = $props();

  let hideChanges = $state(false);
  let filesExpanded = $state(true);

  $effect(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    patchId;

    hideChanges = false;
    filesExpanded = true;
  });

  async function loadHighlightedDiff(rid: string, base: string, head: string) {
    return invoke<Diff>("get_diff", {
      rid,
      options: {
        base,
        head,
        unified: 3,
        highlight: true,
      },
    });
  }

  async function loadCommits(rid: string, base: string, head: string) {
    return invoke<Commit[]>("list_commits", {
      rid,
      base,
      head,
    });
  }
</script>

<style>
  .commits {
    position: relative;
    display: flex;
    flex-direction: column;
    font-size: 0.875rem;
    margin-left: 0.5rem;
    gap: 0.5rem;
    padding: 1rem 0.5rem 0.5rem 1rem;
    border-left: 1px solid var(--color-fill-separator);
  }
  .commit:last-of-type::after {
    content: "";
    position: absolute;
    left: -18.5px;
    top: 14px;
    bottom: -0.5rem;
    border-left: 4px solid var(--color-background-default);
  }
  .commit-dot {
    width: 4px;
    height: 4px;
    position: absolute;
    top: 0.625rem;
    left: -18.5px;
    background-color: var(--color-fill-separator);
  }
  .hide {
    display: none;
  }
</style>

<div
  class="txt-semibold global-flex"
  style:margin-bottom={hideChanges ? undefined : "1rem"}>
  <NakedButton variant="ghost" onclick={() => (hideChanges = !hideChanges)}>
    <Icon name={hideChanges ? "chevron-right" : "chevron-down"} />
    <div class="txt-semibold global-flex txt-regular">Changes</div>
  </NakedButton>
  {#if !hideChanges}
    <div style:margin-left="auto">
      <NakedButton
        variant="ghost"
        onclick={() => (filesExpanded = !filesExpanded)}>
        {#if filesExpanded === true}
          <Icon name="collapse" />
          Collapse all
        {:else}
          <Icon name="expand" />
          Expand all
        {/if}
      </NakedButton>
    </div>
  {/if}
</div>

<div class:hide={hideChanges}>
  {#await loadCommits(rid, revision.base, revision.head) then commits}
    <div style:margin-bottom="1rem">
      <CommitsContainer expanded={filesExpanded}>
        {#snippet leftHeader()}
          <div class="txt-semibold">Commits</div>
        {/snippet}
        {#snippet children()}
          <div style:padding="0 1rem">
            <div
              class="global-flex txt-small"
              style:color="var(--color-foreground-dim)">
              <Icon name="branch" /><Id id={revision.base} variant="commit" />
              <div class="global-counter">base</div>
            </div>
            <div class="commits">
              {#each commits.reverse() as commit}
                <div class="commit" style:position="relative">
                  <div class="commit-dot"></div>
                  <CobCommitTeaser {commit} />
                </div>
              {/each}
            </div>
          </div>
        {/snippet}
      </CommitsContainer>
    </div>
  {/await}

  {#await loadHighlightedDiff(rid, revision.base, revision.head)}
    <span class="txt-small">Loading…</span>
  {:then diff}
    <Changeset
      expanded={filesExpanded}
      head={revision.head}
      {diff}
      {codeComments} />
  {/await}
</div>
