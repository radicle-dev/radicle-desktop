<script lang="ts">
  import Icon from "@app/components/Icon.svelte";
  import Label from "@app/components/Label.svelte";
  import TextInput from "@app/components/TextInput.svelte";

  interface Props {
    allowedToEdit: boolean;
    labels: string[];
    submitInProgress: boolean;
    save: (updatedLabels: string[]) => void;
  }

  const {
    allowedToEdit = false,
    labels,
    submitInProgress = false,
    save,
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
    gap: 0.5rem;
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
      : "Only delegates are allowed to add labels"}
    style:cursor={allowedToEdit ? "pointer" : "default"}
    class="title-button"
    onclick={() => {
      inputValue = "";
      showInput = !showInput;
    }}>
    {#if updatedLabels.length === 0}
      Add labels
    {:else}
      Labels
    {/if}

    {#if !showInput && allowedToEdit}
      <span class="add-icon">
        <Icon name="add"></Icon>
      </span>
    {/if}
  </button>

  {#if allowedToEdit}
    <div class="global-flex edit-icons">
      {#if showInput}
        <Icon
          onclick={addLabel}
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
      placeholder="Add label"
      bind:value={inputValue}
      onSubmit={addLabel} />
    {#if !valid && validationMessage}
      <div class="validation-message">
        <Icon name="warning" />{validationMessage}
      </div>
    {/if}
  </div>
{/if}

{#if updatedLabels.length > 0}
  <div class="body">
    {#if allowedToEdit}
      {#each updatedLabels as label}
        <button
          class="global-counter txt-small"
          style:background-color="var(--color-fill-counter)"
          style:padding="0 0.5rem"
          style:max-width="10rem"
          onclick={() => (removeToggles[label] = !removeToggles[label])}>
          <div class="txt-overflow" title={label}>{label}</div>
          {#if removeToggles[label]}
            <span style:margin-right="0.5rem">
              <Icon name="cross" onclick={() => removeLabel(label)} />
            </span>
          {/if}
        </button>
      {/each}
    {:else}
      {#each updatedLabels as label}
        <Label {label} />
      {/each}
    {/if}
  </div>
{/if}
