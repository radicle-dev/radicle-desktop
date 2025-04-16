<script lang="ts">
  import type { State } from "@bindings/cob/issue/State";

  import capitalize from "lodash/capitalize";
  import isEqual from "lodash/isEqual";

  import { closeFocused } from "@app/components/Popover.svelte";
  import { issueStatusBackgroundColor, issueStatusColor } from "@app/lib/utils";

  import Border from "@app/components/Border.svelte";
  import DropdownList from "@app/components/DropdownList.svelte";
  import DropdownListItem from "@app/components/DropdownListItem.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Popover from "@app/components/Popover.svelte";

  interface Props {
    selectedState: State;
    onSelect: (selectedStatus: State) => void;
  }

  const { selectedState, onSelect }: Props = $props();
</script>

<style>
  button {
    cursor: pointer;
    border: 0;
    background: none;
    margin: 0;
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: var(--font-size-small);
  }
  .badge {
    gap: 0.375rem;
    padding-right: 0.625rem;
  }
</style>

<Popover popoverPadding="0" popoverPositionTop="2rem" popoverPositionLeft="0">
  {#snippet toggle(onclick)}
    <button {onclick}>
      <span
        class="global-counter badge"
        style:color={issueStatusColor[selectedState.status]}
        style:background-color={issueStatusBackgroundColor[
          selectedState.status
        ]}>
        <Icon
          name={selectedState.status === "open"
            ? "issue"
            : `issue-${selectedState.status}`} />
        {capitalize(selectedState.status)}
        {selectedState.status === "closed" ? `as ${selectedState.reason}` : ""}
        <Icon name="chevron-down" />
      </span>
    </button>
  {/snippet}
  {#snippet popover()}
    <Border variant="ghost">
      <DropdownList
        items={[
          { status: "open" },
          { status: "closed", reason: "solved" },
          { status: "closed", reason: "other" },
        ] as State[]}>
        {#snippet item(state)}
          <DropdownListItem
            selected={isEqual(selectedState, state)}
            onclick={() => {
              onSelect(state);
              closeFocused();
            }}>
            <span
              class="global-flex"
              style:color={issueStatusColor[state.status]}>
              <Icon
                name={state.status === "open"
                  ? "issue"
                  : `issue-${state.status}`} />
              {capitalize(state.status)}
              {state.status === "closed" ? `as ${state.reason}` : ""}
            </span>
          </DropdownListItem>
        {/snippet}
      </DropdownList>
    </Border>
  {/snippet}
</Popover>
