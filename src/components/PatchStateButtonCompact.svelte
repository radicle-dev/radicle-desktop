<script lang="ts">
  import type { State } from "@bindings/cob/patch/State";
  import type { ComponentProps } from "svelte";

  import capitalize from "lodash/capitalize";

  import { patchStatusBackgroundColor, patchStatusColor } from "@app/lib/utils";

  import Border from "@app/components/Border.svelte";
  import DropdownList from "@app/components/DropdownList.svelte";
  import DropdownListItem from "@app/components/DropdownListItem.svelte";
  import Icon from "@app/components/Icon.svelte";
  import { closeFocused } from "@app/components/Popover.svelte";
  import Popover from "@app/components/Popover.svelte";

  interface Props {
    selectedState: State;
    onSelect: (newState: State) => void;
  }

  const { selectedState, onSelect }: Props = $props();
  let focus = $state(false);

  let popoverExpanded: boolean = $state(false);

  function icon(): ComponentProps<typeof Icon>["name"] {
    if (selectedState.status === "merged") {
      return "patch-merged";
    } else if (focus) {
      return "chevron-down";
    } else {
      if (selectedState.status === "open") {
        return "patch";
      } else {
        return `patch-${selectedState.status}`;
      }
    }
  }
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
    height: 2.5rem;
    width: 2.5rem;
    gap: 0.375rem;
    padding-right: 0.625rem;
  }
</style>

<Popover
  popoverPadding="0"
  popoverPositionTop="3rem"
  popoverPositionLeft="0"
  bind:expanded={popoverExpanded}>
  {#snippet toggle(onclick)}
    <button
      onmouseenter={() => (focus = true)}
      onfocus={() => (focus = true)}
      onblur={() => (focus = false)}
      onmouseleave={() => (focus = false)}
      disabled={selectedState.status === "merged"}
      {onclick}
      title={selectedState.status === "merged"
        ? "The state of merged patches can not be changed"
        : "Click to change patch state"}>
      <span
        class="global-counter badge"
        style:color={patchStatusColor[selectedState.status]}
        style:background-color={patchStatusBackgroundColor[
          selectedState.status
        ]}>
        <Icon name={icon()} />
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
