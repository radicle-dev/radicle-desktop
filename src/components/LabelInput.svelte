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
  .header {
    font-size: var(--font-size-small);
    margin-bottom: 0.5rem;
    color: var(--color-foreground-dim);
  }
  .body {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    flex-direction: row;
    gap: 0.5rem;
    font-size: var(--font-size-small);
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
    color: var(--color-foreground-default);
    gap: 0.5rem;
  }
</style>

<div style:width="100%">
  <div class="global-flex" style:align-items="flex-start">
    <div class="header">Labels</div>

    {#if allowedToEdit}
      <div class="global-flex" style:margin-left="auto">
        {#if showInput}
          <Icon
            onclick={addLabel}
            disabled={!valid || inputValue === ""}
            name="checkmark" />
          <Icon
            onclick={() => {
              inputValue = "";
              showInput = false;
            }}
            name="cross" />
        {:else}
          <Icon name="add" onclick={() => (showInput = true)}></Icon>
        {/if}
      </div>
    {/if}
  </div>

  <div class="body">
    {#if allowedToEdit}
      {#each updatedLabels as label}
        <button
          class="global-counter txt-small"
          style:max-width="10rem"
          onclick={() => (removeToggles[label] = !removeToggles[label])}>
          <div class="txt-overflow" title={label}>{label}</div>
          {#if removeToggles[label]}
            <Icon name="cross" onclick={() => removeLabel(label)} />
          {/if}
        </button>
      {/each}
      {#if updatedLabels.length === 0 && !showInput}
        <div class="txt-missing">No labels.</div>
      {/if}
    {:else}
      {#each updatedLabels as label}
        <Label {label} />
      {:else}
        <div class="txt-missing">No labels.</div>
      {/each}
    {/if}
  </div>

  {#if showInput}
    <div style:margin-top="0.5rem">
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
</div>
