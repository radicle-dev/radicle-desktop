<script lang="ts">
  import Button from "@app/components/Button.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Label from "@app/components/Label.svelte";
  import TextInput from "@app/components/TextInput.svelte";

  interface Props {
    allowedToEdit: boolean;
    labels: string[];
    submitInProgress: boolean;
    save: (updatedLabels: string[]) => void;
    preview?: boolean;
  }

  const {
    allowedToEdit = false,
    labels,
    submitInProgress = false,
    save,
    preview = false,
  }: Props = $props();

  let updatedLabels: string[] = $state([]);
  let showInput: boolean = $state(false);
  let inputValue = $state("");
  let validationMessage: string | undefined = $state();
  let valid: boolean = $state(false);

  const sanitizedValue = $derived(inputValue.trim());

  let removeToggles: Record<string, boolean> = $state({});

  $effect(() => {
    // Reset component state whenever the labels change in the parent. This
    // happens when the issue ID changes for example when the user navigates
    // to a different issue via the sidebar.
    updatedLabels = labels;

    showInput = false;
    validationMessage = undefined;
    valid = true;
    removeToggles = {};
  });

  $effect(() => {
    if (inputValue !== "") {
      if (sanitizedValue.length > 0) {
        if (updatedLabels.includes(sanitizedValue)) {
          valid = false;
          validationMessage = "This label is already assigned";
        } else {
          valid = true;
          validationMessage = undefined;
        }
      }
    } else {
      valid = true;
      validationMessage = "";
    }
  });

  function addLabel() {
    if (valid && sanitizedValue) {
      updatedLabels = [...updatedLabels, sanitizedValue].sort();
      inputValue = "";
      save($state.snapshot(updatedLabels));
      showInput = false;
    }
  }

  function removeLabel(label: string) {
    updatedLabels = updatedLabels.filter(x => x !== label);
    save($state.snapshot(updatedLabels));
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
  .removable-label {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
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
      <Icon name="label" />
      {#if updatedLabels.length === 0}
        Add labels
      {:else}
        Labels
      {/if}
    </Button>
    {#each updatedLabels as label}
      <Label {label} />
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
            placeholder="Add label"
            bind:value={inputValue}
            onSubmit={addLabel} />
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
        title={allowedToEdit
          ? undefined
          : "Only delegates are allowed to add labels"}
        onclick={() => {
          inputValue = "";
          showInput = true;
        }}>
        <Icon name="label" />
        {#if updatedLabels.length === 0}
          Add labels
        {:else}
          Labels
        {/if}
      </Button>
    {/if}

    {#each updatedLabels as label}
      {#if allowedToEdit}
        <button
          class="removable-label"
          onclick={() => (removeToggles[label] = !removeToggles[label])}>
          <Label {label} />
          {#if removeToggles[label]}
            <Icon name="close" onclick={() => removeLabel(label)} />
          {/if}
        </button>
      {:else}
        <Label {label} />
      {/if}
    {/each}
  </div>

  {#if !valid && validationMessage}
    <div class="validation-message">
      <Icon name="warning" />{validationMessage}
    </div>
  {/if}
{/if}
