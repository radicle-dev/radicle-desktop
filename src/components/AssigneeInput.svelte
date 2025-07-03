<script lang="ts">
  import type { Author } from "@bindings/cob/Author";

  import { invoke } from "@app/lib/invoke";
  import {
    authorForNodeId,
    parseNodeId,
    publicKeyFromDid,
  } from "@app/lib/utils";

  import Icon from "@app/components/Icon.svelte";
  import NodeId from "@app/components/NodeId.svelte";
  import TextInput from "@app/components/TextInput.svelte";

  interface Props {
    allowedToEdit: boolean;
    assignees: Author[];
    submitInProgress: boolean;
    save: (updatedAssignees: Author[]) => void;
  }

  const {
    allowedToEdit = false,
    assignees = $bindable(),
    submitInProgress = false,
    save,
  }: Props = $props();

  let updatedAssignees: Author[] = $state([]);
  let showInput: boolean = $state(false);
  let inputValue = $state("");
  let validationMessage: string | undefined = $state(undefined);
  let valid: boolean = $state(false);
  let assignee: string | undefined = undefined;

  let removeToggles: Record<string, boolean> = $state({});

  $effect(() => {
    // Reset component state whenever the assignees change in the parent. This
    // happens when the issue ID changes for example when the user navigates
    // to a different issue via the sidebar.
    updatedAssignees = assignees;

    showInput = false;
    validationMessage = undefined;
    valid = true;
    removeToggles = {};
  });

  $effect(() => {
    if (inputValue === "") {
      validationMessage = "";
      valid = true;
    } else {
      const parsedNodeId = parseNodeId(inputValue);
      if (parsedNodeId) {
        assignee = `${parsedNodeId.prefix}${parsedNodeId.pubkey}`;
        if (updatedAssignees.find(({ did }) => did === assignee)) {
          validationMessage = "This assignee is already added";
          valid = false;
        } else {
          validationMessage = undefined;
          valid = true;
        }
      } else {
        validationMessage = "This is not a valid DID";
        valid = false;
      }
    }
  });

  async function addAssignee() {
    if (valid && assignee) {
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
  .add-icon {
    display: none;
  }
  .title-button:hover .add-icon {
    display: flex;
  }
  .title-button {
    font-size: var(--font-size-small);
    color: var(--color-foreground-dim);
  }
  .body {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    flex-direction: row;
    gap: 1rem;
    font-size: var(--font-size-small);
    margin-top: 1rem;
  }
  .validation-message {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    color: var(--color-foreground-red);
    position: relative;
    margin-top: 0.5rem;
  }
  button {
    border: 0;
    cursor: pointer;
    gap: 0.5rem;
    background-color: transparent;
    border: none;
    display: flex;
    color: var(--color-foreground-default);
    padding: 0;
    align-items: center;
  }
</style>

<div class="global-flex">
  <button
    disabled={!allowedToEdit}
    style:color={allowedToEdit
      ? "var(--color-foreground-dim)"
      : "var(--color-foreground-disabled)"}
    title={allowedToEdit
      ? undefined
      : "Only delegates are allowed to add assignees"}
    style:cursor={allowedToEdit ? "pointer" : "default"}
    class="title-button"
    onclick={() => {
      inputValue = "";
      showInput = !showInput;
    }}>
    {#if updatedAssignees.length === 0}
      Add assignees
    {:else}
      Assignees
    {/if}

    {#if !showInput && allowedToEdit}
      <span class="add-icon">
        <Icon name="add" />
      </span>
    {/if}
  </button>

  {#if allowedToEdit}
    <div class="global-flex edit-icons">
      {#if showInput}
        <Icon
          onclick={addAssignee}
          name="checkmark"
          disabled={!valid || inputValue === ""} />
        <Icon
          onclick={() => {
            inputValue = "";
            showInput = false;
          }}
          name="cross" />
      {/if}
    </div>
  {/if}
</div>

{#if showInput}
  <div style:margin-top="1rem">
    <TextInput
      autofocus
      {valid}
      disabled={submitInProgress}
      placeholder="Assignee DID, e.g. did:key:z6MkwPUeUS2…"
      bind:value={inputValue}
      onSubmit={addAssignee} />
    {#if !valid && validationMessage}
      <div class="validation-message">
        <Icon name="warning" />{validationMessage}
      </div>
    {/if}
  </div>
{/if}

{#if updatedAssignees.length > 0}
  <div class="body">
    {#if allowedToEdit}
      {#each updatedAssignees as assignee}
        <button
          class="txt-small"
          onclick={() =>
            (removeToggles[assignee.did] = !removeToggles[assignee.did])}>
          <NodeId {...authorForNodeId(assignee)} />
          {#if removeToggles[assignee.did]}
            <Icon name="cross" onclick={() => removeAssignee(assignee)} />
          {/if}
        </button>
      {/each}
    {:else}
      {#each updatedAssignees as assignee}
        <NodeId {...authorForNodeId(assignee)} />
      {/each}
    {/if}
  </div>
{/if}
