<script lang="ts">
  import type { Author } from "@bindings/cob/Author";
  import type { Config } from "@bindings/config/Config";
  import type { Patch } from "@bindings/cob/patch/Patch";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import * as roles from "@app/lib/roles";
  import { announce } from "@app/components/AnnounceSwitch.svelte";
  import { invoke } from "@app/lib/invoke";
  import { nodeRunning } from "@app/lib/events";

  import AssigneeInput from "@app/components/AssigneeInput.svelte";
  import LabelInput from "@app/components/LabelInput.svelte";
  import NodeId from "./NodeId.svelte";
  import { authorForNodeId } from "@app/lib/utils";
  import PatchStateButton from "./PatchStateButton.svelte";

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
  .metadata-section {
    padding: 0.5rem;
    font-size: var(--font-size-small);
    display: flex;
    flex-direction: column;
    align-items: flex;
    height: 100%;
  }
  .metadata-section-title {
    margin-bottom: 0.5rem;
    color: var(--color-foreground-dim);
  }
</style>

<div
  class="global-flex"
  style:flex-direction={horizontal ? "row" : "column"}
  style:align-items="flex-start">
  <div
    class="metadata-section"
    style={horizontal ? "flex: 1;" : "width: 100%;"}>
    <div class="metadata-section-title">Author</div>
    <NodeId {...authorForNodeId(patch.author)} />
  </div>

  <div
    class="metadata-section"
    style={horizontal ? "flex: 1;" : "width: 100%;"}>
    <div class="metadata-section-title">Status</div>
    <PatchStateButton
      selectedState={patch.state}
      onSelect={newState => {
        void saveState(newState);
      }} />
  </div>

  <div
    class="metadata-section"
    style={horizontal ? "flex: 1;" : "width: 100%;"}>
    <LabelInput
      allowedToEdit={!!roles.isDelegateOrAuthor(
        config.publicKey,
        repo.delegates.map(delegate => delegate.did),
        patch.author.did,
      )}
      labels={patch.labels}
      submitInProgress={labelSaveInProgress}
      save={saveLabels} />
  </div>

  <div
    class="metadata-section"
    style={horizontal ? "flex: 1;" : "width: 100%;"}>
    <AssigneeInput
      allowedToEdit={!!roles.isDelegateOrAuthor(
        config.publicKey,
        repo.delegates.map(delegate => delegate.did),
        patch.author.did,
      )}
      assignees={patch.assignees}
      submitInProgress={assigneesSaveInProgress}
      save={saveAssignees} />
  </div>
</div>
