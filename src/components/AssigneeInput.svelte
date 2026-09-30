<script lang="ts">
  import type { Author } from "@bindings/cob/Author";

  import { parseAssignee } from "@app/lib/inputValidation";
  import { invoke } from "@app/lib/invoke";
  import { authorForNodeId, publicKeyFromDid } from "@app/lib/utils";

  import Button from "@app/components/Button.svelte";
  import Icon from "@app/components/Icon.svelte";
  import NodeId from "@app/components/NodeId.svelte";
  import TextInput from "@app/components/TextInput.svelte";

  interface Props {
    allowedToEdit: boolean;
    assignees: Author[];
    submitInProgress: boolean;
    save: (updatedAssignees: Author[]) => void;
    preview?: boolean;
  }

  const {
    allowedToEdit = false,
    assignees = $bindable(),
    submitInProgress = false,
    save,
    preview = false,
  }: Props = $props();

  let updatedAssignees: Author[] = $state([]);
  let showInput: boolean = $state(false);
  let inputValue = $state("");
  const parsed = $derived(parseAssignee(inputValue, updatedAssignees));
  const validationMessage = $derived(parsed.error);
  const valid = $derived(parsed.error === undefined);

  let removeToggles: Record<string, boolean> = $state({});

  $effect(() => {
    // Reset component state whenever the assignees change in the parent. This
    // happens when the issue ID changes for example when the user navigates
    // to a different issue via the sidebar.
    updatedAssignees = assignees;

    showInput = false;
    removeToggles = {};
  });

  async function addAssignee() {
    const assignee = parsed.did;
    if (assignee) {
      const alias = await invoke<string | null>("alias", {
        nid: publicKeyFromDid(assignee),
      });
      updatedAssignees = [
        ...updatedAssignees,
        { did: assignee, alias: alias ?? undefined },
      ];
      inputValue = "";
      save($state.snapshot(updatedAssignees));
      showInput = false;
    }
  }

  function removeAssignee(assignee: Author) {
    updatedAssignees = updatedAssignees.filter(
      ({ did }) => did !== assignee.did,
    );
    save($state.snapshot(updatedAssignees));
    showInput = false;
  }
</script>

<style>
  .row {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.5rem;
  }
  .validation-message {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    color: var(--color-feedback-error-text);
    position: relative;
    margin-top: 0.5rem;
  }
  .input-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  button {
    border: 0;
    cursor: pointer;
    gap: 0.5rem;
    background-color: transparent;
    border: none;
    display: flex;
    color: var(--color-text-secondary);
    padding: 0;
    align-items: center;
  }
</style>

{#if preview}
  <div class="row">
    <Button variant="outline" disabled>
      <Icon name="avatar-incognito" />
      {#if updatedAssignees.length === 0}
        Add assignees
      {:else}
        Assignees
      {/if}
    </Button>
    {#each updatedAssignees as assignee}
      <span style:color="var(--color-text-secondary)">
        <NodeId {...authorForNodeId(assignee)} />
      </span>
    {/each}
  </div>
{:else}
  <div class="row">
    {#if showInput}
      <div class="input-row">
        <div style:flex="1" style:min-width="0">
          <TextInput
            autofocus
            {valid}
            disabled={submitInProgress}
            placeholder="Assignee DID, e.g. did:key:z6MkwPUeUS2…"
            bind:value={inputValue}
            onSubmit={addAssignee} />
        </div>
        <Button
          variant="outline"
          onclick={() => {
            showInput = false;
            inputValue = "";
          }}>
          <Icon name="close" />
        </Button>
      </div>
    {:else}
      <Button
        variant="outline"
        disabled={!allowedToEdit}
        title={allowedToEdit ? undefined : "Only delegates can add assignees"}
        onclick={() => {
          inputValue = "";
          showInput = true;
        }}>
        <Icon name="avatar-incognito" />
        {#if updatedAssignees.length === 0}
          Add assignees
        {:else}
          Assignees
        {/if}
      </Button>
    {/if}

    {#if allowedToEdit}
      {#each updatedAssignees as assignee}
        <button
          class="txt-body-m-regular"
          onclick={() =>
            (removeToggles[assignee.did] = !removeToggles[assignee.did])}>
          <NodeId {...authorForNodeId(assignee)} />
          {#if removeToggles[assignee.did]}
            <Icon name="close" onclick={() => removeAssignee(assignee)} />
          {/if}
        </button>
      {/each}
    {:else}
      {#each updatedAssignees as assignee}
        <NodeId {...authorForNodeId(assignee)} />
      {/each}
    {/if}
  </div>

  {#if !valid && validationMessage}
    <div class="validation-message">
      <Icon name="warning" />{validationMessage}
    </div>
  {/if}
{/if}
