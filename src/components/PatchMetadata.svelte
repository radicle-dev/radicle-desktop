<script lang="ts">
  import type { Author } from "@bindings/cob/Author";
  import type { Patch } from "@bindings/cob/patch/Patch";
  import type { Config } from "@bindings/config/Config";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import { nodeRunning } from "@app/lib/events";
  import { invoke } from "@app/lib/invoke";
  import { show } from "@app/lib/modal";
  import * as roles from "@app/lib/roles";

  import { announce } from "@app/components/AnnounceSwitch.svelte";
  import AssigneeChip from "@app/components/AssigneeChip.svelte";
  import Button from "@app/components/Button.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Label from "@app/components/Label.svelte";
  import PatchStateButton from "@app/components/PatchStateButton.svelte";
  import AssigneeModal from "@app/modals/AssigneeModal.svelte";
  import LabelModal from "@app/modals/LabelModal.svelte";

  interface Props {
    config: Config;
    horizontal?: boolean;
    loadPatch: () => Promise<void>;
    patch: Patch;
    repo: RepoInfo;
    saveState: (newState: Patch["state"]) => Promise<void>;
  }

  const {
    config,
    horizontal = false,
    loadPatch,
    patch,
    repo,
    saveState,
  }: Props = $props();

  const canEdit = $derived(
    !!roles.isDelegate(
      config.publicKey,
      repo.delegates.map(delegate => delegate.did),
    ),
  );

  async function saveLabels(labels: string[]) {
    try {
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
      await loadPatch();
    }
  }

  async function saveAssignees(assignees: Author[]) {
    try {
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
      await loadPatch();
    }
  }

  function removeLabel(label: string) {
    void saveLabels(patch.labels.filter(l => l !== label));
  }
  function removeAssignee(did: string) {
    void saveAssignees(patch.assignees.filter(a => a.did !== did));
  }

  function openLabels() {
    show({
      component: LabelModal,
      props: { labels: patch.labels, rid: repo.rid, save: saveLabels },
    });
  }
  function openAssignees() {
    show({
      component: AssigneeModal,
      props: { assignees: patch.assignees, save: saveAssignees },
    });
  }
</script>

<style>
  .metadata-section {
    padding: 0.5rem;
    font: var(--txt-body-m-regular);
    display: flex;
    flex-direction: column;
    align-items: flex;
    height: 100%;
  }
  .meta-group {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
  }
</style>

<div
  class="global-flex"
  style:flex-direction={horizontal ? "row" : "column"}
  style:align-items="flex-start">
  <div
    class="metadata-section"
    style={horizontal ? "flex: 1;" : "width: 100%;"}>
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
  </div>

  <div
    class="metadata-section"
    style={horizontal ? "flex: 1;" : "width: 100%;"}>
    <div class="meta-group">
      {#each patch.labels as label (label)}
        <Label
          {label}
          styleHeight="2rem"
          onRemove={canEdit ? () => removeLabel(label) : undefined} />
      {/each}
      <Button
        variant="outline"
        disabled={!canEdit}
        title={canEdit ? undefined : "Only delegates can add labels"}
        onclick={openLabels}>
        <Icon name="label" />
        Add labels
      </Button>
    </div>
  </div>

  <div
    class="metadata-section"
    style={horizontal ? "flex: 1;" : "width: 100%;"}>
    <div class="meta-group">
      {#each patch.assignees as assignee (assignee.did)}
        <AssigneeChip
          {assignee}
          onRemove={canEdit ? () => removeAssignee(assignee.did) : undefined} />
      {/each}
      <Button
        variant="outline"
        disabled={!canEdit}
        title={canEdit ? undefined : "Only delegates can add assignees"}
        onclick={openAssignees}>
        <Icon name="avatar-incognito" />
        Add assignees
      </Button>
    </div>
  </div>
</div>
