<script lang="ts">
  import type { PatchStatus } from "@app/views/repo/router";
  import type { ProjectPayloadMeta } from "@bindings/repo/ProjectPayloadMeta";

  import capitalize from "lodash/capitalize";

  import { patchStatusColor } from "@app/lib/utils";

  import { closeFocused } from "@app/components/Popover.svelte";

  import Border from "@app/components/Border.svelte";
  import DropdownList from "@app/components/DropdownList.svelte";
  import DropdownListItem from "@app/components/DropdownListItem.svelte";
  import Icon from "@app/components/Icon.svelte";
  import OutlineButton from "@app/components/OutlineButton.svelte";
  import Popover from "@app/components/Popover.svelte";

  interface Props {
    counters: ProjectPayloadMeta["patches"];
    select: (filter: PatchStatus | undefined) => Promise<void>;
    status: PatchStatus | undefined;
  }

  const { status, select, counters }: Props = $props();

  let popoverExpanded: boolean = $state(false);
</script>

{#snippet iconSnippet(status: PatchStatus | undefined)}
  <div class="icon" style:color={status ? patchStatusColor[status] : undefined}>
    <Icon
      name={status === undefined || status === "open"
        ? "patch"
        : `patch-${status}`} />
  </div>
{/snippet}

{#snippet counterSnippet(status: PatchStatus | undefined)}
  <div style:margin-left="auto" style:padding-left="0.25rem">
    {#if status}
      {counters[status]}
    {:else}
      {counters.draft + counters.open + counters.archived + counters.merged}
    {/if}
  </div>
{/snippet}

<Popover
  popoverPositionLeft="0"
  popoverPositionTop="3rem"
  bind:expanded={popoverExpanded}>
  {#snippet toggle(onclick)}
    <OutlineButton
      variant="ghost"
      {onclick}
      styleHeight="2.5rem"
      active={popoverExpanded}>
      {@render iconSnippet(status)}
      {status ? capitalize(status) : "All"}
      {@render counterSnippet(status)}
      <Icon name={popoverExpanded ? "chevron-up" : "chevron-down"} />
    </OutlineButton>
  {/snippet}

  {#snippet popover()}
    <Border variant="ghost">
      <DropdownList
        items={[undefined, "draft", "open", "archived", "merged"] as const}>
        {#snippet item(state)}
          <DropdownListItem
            styleGap="0.5rem"
            styleMinHeight="2.5rem"
            selected={status === state}
            onclick={async () => {
              await select(state);
              closeFocused();
            }}>
            {@render iconSnippet(state)}
            {state ? capitalize(state) : "All"}
            {@render counterSnippet(state)}
          </DropdownListItem>
        {/snippet}
      </DropdownList>
    </Border>
  {/snippet}
</Popover>
