<script lang="ts">
  import Icon from "@app/components/Icon.svelte";
  import InlineTitle from "@app/components/InlineTitle.svelte";
  import NakedButton from "@app/components/NakedButton.svelte";
  import TextInput from "@app/components/TextInput.svelte";

  interface Props {
    allowedToEdit: true | undefined;
    cobId: string;
    title: string;
    updateTitle: (newTitle: string) => Promise<void>;
  }

  const { allowedToEdit, cobId, title, updateTitle }: Props = $props();

  let editingTitle = $state(false);
  let newTitle = $state("");

  $effect(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    cobId;

    editingTitle = false;
    newTitle = title;
  });

  async function save() {
    if (newTitle.trim().length <= 0) {
      return;
    }

    if (title === newTitle) {
      editingTitle = false;
      return;
    }

    await updateTitle(newTitle);
  }
</script>

<style>
  .title {
    font-size: var(--font-size-medium);
    font-weight: var(--font-weight-medium);
    -webkit-user-select: text;
    user-select: text;
    display: flex;
    align-items: center;
    word-break: break-word;
    min-height: 2.5rem;
    width: 100%;
  }
  .edit-title-icon {
    display: none;
  }
  .title-wrapper:hover .edit-title-icon {
    display: flex;
  }
</style>

<div class="title">
  {#if editingTitle}
    <TextInput
      valid={newTitle.trim().length > 0}
      bind:value={newTitle}
      autofocus
      onSubmit={save}
      onDismiss={() => {
        newTitle = title;
        editingTitle = false;
      }} />
    <div class="global-flex" style:margin-left="0.5rem">
      <NakedButton
        variant="ghost"
        styleHeight="2.5rem"
        disabled={!(newTitle.trim().length > 0)}
        onclick={save}>
        <Icon name="checkmark" />
      </NakedButton>
      <NakedButton
        variant="ghost"
        styleHeight="2.5rem"
        onclick={() => {
          newTitle = title;
          editingTitle = false;
        }}>
        <Icon name="cross" />
      </NakedButton>
    </div>
  {:else}
    <div class="global-flex" style:gap="0">
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div
        class="title-wrapper global-flex"
        role="button"
        style:cursor={allowedToEdit ? "pointer" : "default"}
        onclick={() => {
          if (allowedToEdit) {
            editingTitle = true;
          }
        }}
        tabindex="0">
        <InlineTitle content={title} fontSize="medium" />
        {#if allowedToEdit}
          <div class="edit-title-icon"><Icon name="pen" /></div>
        {/if}
      </div>
    </div>
  {/if}
</div>
