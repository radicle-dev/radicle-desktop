<script lang="ts">
  import type { IssueStatus } from "@app/views/repo/router";
  import type { ProjectPayloadMeta } from "@bindings/repo/ProjectPayloadMeta";

  import { closeFocused } from "./Popover.svelte";

  import capitalize from "lodash/capitalize";
  import { issueStatusColor } from "@app/lib/utils";

  import Border from "./Border.svelte";
  import DropdownList from "./DropdownList.svelte";
  import DropdownListItem from "./DropdownListItem.svelte";
  import Icon from "./Icon.svelte";
  import OutlineButton from "./OutlineButton.svelte";
  import Popover from "./Popover.svelte";

  interface Props {
    changeFilter: (status: IssueStatus) => void;
    status: IssueStatus;
    counters: ProjectPayloadMeta["issues"];
  }

  const { changeFilter, counters, status }: Props = $props();
</script>

{#snippet iconSnippet(status: IssueStatus)}
  <div
    class="icon"
    style:color={status === "all" ? undefined : issueStatusColor[status]}>
    <Icon name={status === "closed" ? "issue-closed" : "issue"} />
  </div>
{/snippet}

{#snippet counterSnippet(status: IssueStatus)}
  <div style:margin-left="auto" style:padding-left="0.25rem">
    {#if status === "all"}
      {counters.open + counters.closed}
    {:else}
      {counters[status]}
    {/if}
  </div>
{/snippet}

<Popover popoverPositionLeft="0" popoverPositionTop="3rem">
  {#snippet toggle(onclick)}
    <OutlineButton variant="ghost" {onclick} styleHeight="2.5rem">
      {@render iconSnippet(status)}
      {capitalize(status)}
      {@render counterSnippet(status)}
      <Icon name="chevron-down" />
    </OutlineButton>
  {/snippet}

  {#snippet popover()}
    <Border variant="ghost">
      <DropdownList items={["all", "open", "closed"] as IssueStatus[]}>
        {#snippet item(state)}
          <DropdownListItem
            styleGap="0.5rem"
            styleMinHeight="2.5rem"
            selected={status === state}
            onclick={() => {
              changeFilter(state);
              closeFocused();
            }}>
            {@render iconSnippet(state)}
            {capitalize(state)}
            {@render counterSnippet(state)}
          </DropdownListItem>
        {/snippet}
      </DropdownList>
    </Border>
  {/snippet}
</Popover>
