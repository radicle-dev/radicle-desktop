<script lang="ts">
  import type { Author } from "@bindings/cob/Author";
  import type { Issue } from "@bindings/cob/issue/Issue";
  import type { Config } from "@bindings/config/Config";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import debounce from "lodash/debounce";

  import { nodeRunning } from "@app/lib/events";
  import { invoke, writeToClipboard } from "@app/lib/invoke";
  import * as roles from "@app/lib/roles";
  import { authorForNodeId, formatOid } from "@app/lib/utils";

  import { announce } from "@app/components/AnnounceSwitch.svelte";
  import AssigneeInput from "@app/components/AssigneeInput.svelte";
  import Icon from "@app/components/Icon.svelte";
  import LabelInput from "@app/components/LabelInput.svelte";
  import NodeId from "@app/components/NodeId.svelte";

  interface Props {
    config: Config;
    issue: Issue;
    repo: RepoInfo;
    reload: () => Promise<void>;
  }

  const { config, issue, repo, reload }: Props = $props();

  let issueIdCopied = $state(false);
  const resetIssueIdCopied = debounce(() => {
    issueIdCopied = false;
  }, 1000);
  async function copyIssueId() {
    await writeToClipboard(issue.id);
    issueIdCopied = true;
    resetIssueIdCopied();
  }

  let labelSaveInProgress: boolean = $state(false);
  let assigneesSaveInProgress: boolean = $state(false);

  async function saveLabels(labels: string[]) {
    try {
      labelSaveInProgress = true;
      await invoke("edit_issue", {
        rid: repo.rid,
        cobId: issue.id,
        action: { type: "label", labels },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Editing labels failed", error);
    } finally {
      labelSaveInProgress = false;
      await reload();
    }
  }

  async function saveAssignees(assignees: Author[]) {
    try {
      assigneesSaveInProgress = true;
      await invoke("edit_issue", {
        rid: repo.rid,
        cobId: issue.id,
        action: { type: "assign", assignees },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Editing assignees failed", error);
    } finally {
      assigneesSaveInProgress = false;
      await reload();
    }
  }
</script>

<style>
  .meta-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
    font: var(--txt-body-m-regular);
  }
  .author-chip,
  .issue-id-chip {
    display: inline-flex;
    align-items: center;
    height: 2rem;
    padding: 0 0.5rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    background-color: var(--color-surface-canvas);
    color: var(--color-text-tertiary);
    font: var(--txt-body-m-regular);
  }
  .issue-id-chip {
    gap: 0.375rem;
    cursor: pointer;
  }
  .issue-id-chip:hover,
  .issue-id-chip:focus-visible {
    background-color: var(--color-surface-subtle);
    color: var(--color-text-primary);
  }
  .iid-icon-value {
    font: var(--txt-code-regular);
  }
  /* Hash icon by default, copy icon on hover, checkmark on click. */
  .iid-icon-default,
  .iid-icon-hover {
    display: inline-flex;
    align-items: center;
  }
  .iid-icon-hover {
    display: none;
  }
  .issue-id-chip:hover .iid-icon-default,
  .issue-id-chip:focus-visible .iid-icon-default {
    display: none;
  }
  .issue-id-chip:hover .iid-icon-hover,
  .issue-id-chip:focus-visible .iid-icon-hover {
    display: inline-flex;
  }
</style>

<div class="meta-row">
  <div class="author-chip" title="Issue author">
    <NodeId {...authorForNodeId(issue.author)} />
  </div>
  <button
    type="button"
    class="issue-id-chip"
    title={issueIdCopied ? "Copied to clipboard" : "Copy issue ID"}
    onclick={copyIssueId}>
    {#if issueIdCopied}
      <Icon name="checkmark" />
    {:else}
      <span class="iid-icon-default"><Icon name="hash" /></span>
      <span class="iid-icon-hover"><Icon name="copy" /></span>
    {/if}
    <span class="iid-icon-value">{formatOid(issue.id)}</span>
  </button>
  <LabelInput
    allowedToEdit={!!roles.isDelegate(
      config.publicKey,
      repo.delegates.map(delegate => delegate.did),
    )}
    labels={issue.labels}
    submitInProgress={labelSaveInProgress}
    save={saveLabels} />
  <AssigneeInput
    allowedToEdit={!!roles.isDelegate(
      config.publicKey,
      repo.delegates.map(delegate => delegate.did),
    )}
    assignees={issue.assignees}
    submitInProgress={assigneesSaveInProgress}
    save={saveAssignees} />
</div>
