<script lang="ts">
  import type { ComponentProps } from "svelte";

  import { saveToDisk, writeToClipboard } from "@app/lib/invoke";

  import DropdownList from "@app/components/DropdownList.svelte";
  import DropdownListItem from "@app/components/DropdownListItem.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Popover, { closeFocused } from "@app/components/Popover.svelte";

  interface Props {
    // Built lazily so large diff text isn't materialized until an action runs.
    text: () => string;
    // Default filename offered when saving to disk.
    fileName: string;
    title?: string;
  }

  const { text, fileName, title = "Diff actions" }: Props = $props();

  let expanded = $state(false);

  type Action = {
    label: string;
    icon: ComponentProps<typeof Icon>["name"];
    run: () => Promise<void>;
  };

  const actions: Action[] = [
    {
      label: "Copy to clipboard",
      icon: "copy",
      run: () => writeToClipboard(text()),
    },
    {
      label: "Save to disk",
      icon: "download",
      run: () => saveToDisk(fileName, text()),
    },
  ];

  async function run(action: Action) {
    closeFocused();
    try {
      await action.run();
    } catch (e) {
      console.error(`${action.label} failed`, e);
    }
  }
</script>

<style>
  .trigger {
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    color: var(--color-text-secondary);
    display: flex;
    align-items: center;
  }
  .trigger:hover,
  .trigger.active {
    color: var(--color-text-primary);
  }
  .menu {
    padding: 0.25rem;
    background-color: var(--color-surface-canvas);
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
  }
</style>

<Popover placement="bottom-end" bind:expanded>
  {#snippet toggle(onclick)}
    <button
      class="trigger"
      class:active={expanded}
      {title}
      {onclick}
      aria-label={title}>
      <Icon name="ellipsis-vertical" />
    </button>
  {/snippet}

  {#snippet popover()}
    <div class="menu">
      <DropdownList items={actions}>
        {#snippet item(action)}
          <DropdownListItem
            selected={false}
            styleGap="0.5rem"
            onclick={() => run(action)}>
            <Icon name={action.icon} />
            {action.label}
          </DropdownListItem>
        {/snippet}
      </DropdownList>
    </div>
  {/snippet}
</Popover>
