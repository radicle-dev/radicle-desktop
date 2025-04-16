<script lang="ts">
  import type { State } from "@bindings/cob/patch/State";

  import capitalize from "lodash/capitalize";

  import { closeFocused } from "@app/components/Popover.svelte";
  import { patchStatusBackgroundColor, patchStatusColor } from "@app/lib/utils";

  import Border from "@app/components/Border.svelte";
  import DropdownList from "@app/components/DropdownList.svelte";
  import DropdownListItem from "@app/components/DropdownListItem.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Popover from "@app/components/Popover.svelte";

  interface Props {
    selectedState: State;
    onSelect: (newState: State) => void;
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
  button:disabled {
    cursor: inherit;
  }

  .badge {
    gap: 0.375rem;
    padding-right: 0.625rem;
  }
</style>

<Popover popoverPadding="0" popoverPositionTop="2rem" popoverPositionLeft="0">
  {#snippet toggle(onclick)}
    <button disabled={selectedState.status === "merged"} {onclick}>
      <span
        class="global-counter badge"
        style:color={patchStatusColor[selectedState.status]}
        style:background-color={patchStatusBackgroundColor[
          selectedState.status
        ]}>
        <Icon
          name={selectedState.status === "open"
            ? "patch"
            : `patch-${selectedState.status}`} />
        {capitalize(selectedState.status)}
        {#if selectedState.status !== "merged"}
          <Icon name="chevron-down" />
        {/if}
      </span>
    </button>
  {/snippet}
  {#snippet popover()}
    <Border variant="ghost">
      <DropdownList
        items={[
          { status: "open" },
          { status: "draft" },
          { status: "archived" },
        ] as State[]}>
        {#snippet item(state)}
          <DropdownListItem
            selected={selectedState.status === state.status}
            onclick={() => {
              onSelect(state);
              closeFocused();
            }}>
            <span
              class="global-flex"
              style:color={patchStatusColor[state.status]}>
              <Icon
                name={state.status === "open"
                  ? "patch"
                  : `patch-${state.status}`} />
              {capitalize(state.status)}
            </span>
          </DropdownListItem>
        {/snippet}
      </DropdownList>
    </Border>
  {/snippet}
</Popover>
