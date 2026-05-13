<script lang="ts">
  import type { Author } from "@bindings/cob/Author";
  import type { Patch } from "@bindings/cob/patch/Patch";
  import type { Config } from "@bindings/config/Config";
  import type { Stats } from "@bindings/diff/Stats";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import { nodeRunning } from "@app/lib/events";
  import { invoke } from "@app/lib/invoke";
  import * as roles from "@app/lib/roles";
  import { pluralize } from "@app/lib/utils";

  import { announce } from "@app/components/AnnounceSwitch.svelte";
  import AssigneeInput from "@app/components/AssigneeInput.svelte";
  import Icon from "@app/components/Icon.svelte";
  import LabelInput from "@app/components/LabelInput.svelte";
  import PatchStateButton from "@app/components/PatchStateButton.svelte";

  interface Props {
    config: Config;
    loadPatch: () => Promise<void>;
    patch: Patch;
    repo: RepoInfo;
    saveState: (newState: Patch["state"]) => Promise<void>;
    stats?: Stats;
  }

  const { config, loadPatch, patch, repo, saveState, stats }: Props = $props();

  let labelSaveInProgress: boolean = $state(false);
  let assigneesSaveInProgress: boolean = $state(false);

  async function saveLabels(labels: string[]) {
    try {
      labelSaveInProgress = true;
      await invoke("edit_patch", {
        rid: repo.rid,
        cobId: patch.id,
        action: {
          type: "label",
          labels,
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Editing labels failed", error);
    } finally {
      labelSaveInProgress = false;
      await loadPatch();
    }
  }

  async function saveAssignees(assignees: Author[]) {
    try {
      assigneesSaveInProgress = true;
      await invoke("edit_patch", {
        rid: repo.rid,
        cobId: patch.id,
        action: {
          type: "assign",
          assignees,
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Editing assignees failed", error);
    } finally {
      assigneesSaveInProgress = false;
      await loadPatch();
    }
  }
</script>

<style>
  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
    font: var(--txt-body-m-regular);
  }
  .stats {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    height: 2rem;
    padding: 0 0.5rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    background-color: var(--color-surface-canvas);
    color: var(--color-text-tertiary);
  }
  .stats .insertions {
    color: var(--color-feedback-success-text);
  }
  .stats .deletions {
    color: var(--color-feedback-error-text);
  }
</style>

<div class="row">
  {#if stats}
    <div class="stats">
      <Icon name="diff" />
      <span>
        {stats.filesChanged}
        {pluralize("file", stats.filesChanged)}
      </span>
      <span class="insertions">+{stats.insertions}</span>
      <span class="deletions">-{stats.deletions}</span>
    </div>
  {/if}
  <PatchStateButton
    selectedState={patch.state}
    onSelect={newState => {
      void saveState(newState);
    }}
    disabled={!roles.isDelegateOrAuthor(
      config.publicKey,
      repo.delegates.map(d => d.did),
      patch.author.did,
    )} />
  <LabelInput
    allowedToEdit={!!roles.isDelegate(
      config.publicKey,
      repo.delegates.map(delegate => delegate.did),
    )}
    labels={patch.labels}
    submitInProgress={labelSaveInProgress}
    save={saveLabels} />
  <AssigneeInput
    allowedToEdit={!!roles.isDelegate(
      config.publicKey,
      repo.delegates.map(delegate => delegate.did),
    )}
    assignees={patch.assignees}
    submitInProgress={assigneesSaveInProgress}
    save={saveAssignees} />
</div>
