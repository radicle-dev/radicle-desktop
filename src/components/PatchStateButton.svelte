<script lang="ts">
  import type { State } from "@bindings/cob/patch/State";

  import isEqual from "lodash/isEqual";

  import { closeFocused } from "@app/components/Popover.svelte";

  import Border from "@app/components/Border.svelte";
  import Button from "@app/components/Button.svelte";
  import DropdownList from "@app/components/DropdownList.svelte";
  import DropdownListItem from "@app/components/DropdownListItem.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Popover from "@app/components/Popover.svelte";

  const {
    save,
    patchState,
  }: {
    save: (state: State) => Promise<void>;
    patchState: State;
  } = $props();

  const actions: { caption: string; state: State }[] = [
    {
      caption: "Reopen",
      state: { status: "open" },
    },
    { caption: "Convert to draft", state: { status: "draft" } },
    { caption: "Archive", state: { status: "archived" } },
  ];

  let selectedAction = $state(
    patchState.status === "open" ? actions[1] : actions[0],
  );

  // React to state changes that come from outside of this button.
  $effect(() => {
    selectedAction = patchState.status === "open" ? actions[1] : actions[0];
  });
</script>

<style>
  .main {
    display: flex;
    flex-direction: row;
    justify-content: center;
  }
</style>

<div class="main">
  <Button
    styleHeight="2.5rem"
    variant="secondary"
    flatRight
    onclick={() =>
      void save($state.snapshot(selectedAction["state"]) as State)}>
    {selectedAction["caption"]}
  </Button>

  <Popover
    popoverPadding="0"
    popoverPositionTop="3rem"
    popoverPositionRight="0">
    {#snippet toggle(onclick)}
      <Button styleHeight="2.5rem" flatLeft {onclick} variant="secondary">
        <Icon name="chevron-down" />
      </Button>
    {/snippet}
    {#snippet popover()}
      <Border variant="ghost">
        <DropdownList
          items={actions.filter(a => !isEqual(a.state, patchState))}>
          {#snippet item(action)}
            <DropdownListItem
              styleGap="0.5rem"
              styleMinHeight="2.5rem"
              selected={isEqual(selectedAction, action)}
              onclick={() => {
                selectedAction = action;
                closeFocused();
              }}>
              {action.caption}
            </DropdownListItem>
          {/snippet}
        </DropdownList>
      </Border>
    {/snippet}
  </Popover>
</div>
