<script lang="ts">
  import type { State } from "@bindings/cob/patch/State";

  import capitalize from "lodash/capitalize";

  import { patchStatusBackgroundColor, patchStatusColor } from "@app/lib/utils";

  import Button from "@app/components/Button.svelte";
  import DropdownList from "@app/components/DropdownList.svelte";
  import DropdownListItem from "@app/components/DropdownListItem.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Popover from "@app/components/Popover.svelte";
  import { closeFocused } from "@app/components/Popover.svelte";

  interface Props {
    selectedState: State;
    onSelect: (newState: State) => void;
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
      disabled={selectedState.status === "merged" || disabled}
      {onclick}
      active={popoverExpanded}
      title={selectedState.status === "merged"
        ? "The state of merged patches can not be changed"
        : disabled
          ? "You must be a delegate or the patch author to change the patch state"
          : undefined}>
      <span
        class="global-chip"
        style:padding="0"
        style:margin-left="-0.25rem"
        style:color={selectedState.status === "merged" || disabled
          ? undefined
          : patchStatusColor[selectedState.status]}
        style:background-color={selectedState.status === "merged" || disabled
          ? undefined
          : patchStatusBackgroundColor[selectedState.status]}>
        <Icon
          name={selectedState.status === "open"
            ? "patch"
            : `patch-${selectedState.status}`} />
      </span>
      <span
        style:color={selectedState.status === "merged" || disabled
          ? undefined
          : "var(--color-text-secondary)"}>
        {capitalize(selectedState.status)}
      </span>
      {#if selectedState.status !== "merged"}
        <Icon name={popoverExpanded ? "chevron-up" : "chevron-down"} />
      {/if}
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
          { status: "draft" },
          { status: "archived" },
        ] as State[]}>
        {#snippet item(state)}
          <DropdownListItem
            selected={selectedState.status === state.status}
            styleGap="0.5rem"
            onclick={() => {
              onSelect(state);
              closeFocused();
            }}>
            <span
              class="global-chip"
              style:padding="0"
              style:margin-left="-0.5rem"
              style:color={patchStatusColor[state.status]}
              style:background-color={patchStatusBackgroundColor[state.status]}>
              <Icon
                name={state.status === "open"
                  ? "patch"
                  : `patch-${state.status}`} />
            </span>
            <span style:color="var(--color-text-secondary)">
              {capitalize(state.status)}
            </span>
          </DropdownListItem>
        {/snippet}
      </DropdownList>
    </div>
  {/snippet}
</Popover>
