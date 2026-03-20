<script lang="ts">
  import type { State } from "@bindings/cob/issue/State";

  import capitalize from "lodash/capitalize";
  import isEqual from "lodash/isEqual";

  import { issueStatusBackgroundColor, issueStatusColor } from "@app/lib/utils";

  import Button from "@app/components/Button.svelte";
  import DropdownList from "@app/components/DropdownList.svelte";
  import DropdownListItem from "@app/components/DropdownListItem.svelte";
  import Icon from "@app/components/Icon.svelte";
  import { closeFocused } from "@app/components/Popover.svelte";
  import Popover from "@app/components/Popover.svelte";

  interface Props {
    selectedState: State;
    onSelect: (selectedStatus: State) => void;
    disabled?: boolean;
  }

  const { selectedState, onSelect, disabled = false }: Props = $props();

  let popoverExpanded: boolean = $state(false);
</script>

<Popover
  popoverPadding="0"
  placement="bottom-start"
  bind:expanded={popoverExpanded}>
  {#snippet toggle(onclick)}
    <Button
      variant="outline"
      {disabled}
      {onclick}
      active={popoverExpanded}
      title={disabled
        ? "You must be a delegate to change the issue state"
        : undefined}>
      <span
        class="global-chip"
        style:padding="0"
        style:margin-left="-0.25rem"
        style:color={disabled
          ? undefined
          : issueStatusColor[selectedState.status]}
        style:background-color={disabled
          ? undefined
          : issueStatusBackgroundColor[selectedState.status]}>
        <Icon
          name={selectedState.status === "open"
            ? "issue"
            : `issue-${selectedState.status}`} />
      </span>
      <span style:color={disabled ? undefined : "var(--color-text-secondary)"}>
        {capitalize(selectedState.status)}
        {selectedState.status === "closed" ? `as ${selectedState.reason}` : ""}
      </span>
      <Icon name={popoverExpanded ? "chevron-up" : "chevron-down"} />
    </Button>
  {/snippet}
  {#snippet popover()}
    <div
      style:border="1px solid var(--color-border-subtle)"
      style:border-radius="var(--border-radius-sm)"
      style:display="flex"
      style:gap="0.5rem"
      style:align-items="center"
      style:background-color="var(--color-surface-canvas)">
      <DropdownList
        items={[
          { status: "open" },
          { status: "closed", reason: "solved" },
          { status: "closed", reason: "other" },
        ] as State[]}>
        {#snippet item(state)}
          <DropdownListItem
            selected={isEqual(selectedState, state)}
            styleGap="0.5rem"
            onclick={() => {
              onSelect(state);
              closeFocused();
            }}>
            <span
              class="global-chip"
              style:padding="0"
              style:margin-left="-0.5rem"
              style:color={issueStatusColor[state.status]}
              style:background-color={issueStatusBackgroundColor[state.status]}>
              <Icon
                name={state.status === "open"
                  ? "issue"
                  : `issue-${state.status}`} />
            </span>
            <span style:color="var(--color-text-secondary)">
              {capitalize(state.status)}
              {state.status === "closed" ? `as ${state.reason}` : ""}
            </span>
          </DropdownListItem>
        {/snippet}
      </DropdownList>
    </div>
  {/snippet}
</Popover>
