<script lang="ts">
  import type { State } from "@bindings/cob/issue/State";

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
    issueState,
  }: {
    save: (state: State) => Promise<void>;
    issueState: State;
  } = $props();

  const actions: { caption: string; state: State }[] = [
    { caption: "Reopen", state: { status: "open" } },
    {
      caption: "Close as solved",
      state: { status: "closed", reason: "solved" },
    },
    { caption: "Close as other", state: { status: "closed", reason: "other" } },
  ];

  let selectedAction = $state(
    issueState.status === "open" ? actions[1] : actions[0],
  );

  // React to state changes that come from outside of this button.
  $effect(() => {
    selectedAction = issueState.status === "open" ? actions[1] : actions[0];
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
    variant="secondary"
    flatRight
    onclick={() => void save($state.snapshot(selectedAction["state"]))}>
    {selectedAction["caption"]}
  </Button>

  <Popover
    popoverPadding="0"
    popoverPositionTop="2.5rem"
    popoverPositionRight="0">
    {#snippet toggle(onclick)}
      <Button flatLeft {onclick} variant="secondary">
        <Icon name="chevron-down" />
      </Button>
    {/snippet}
    {#snippet popover()}
      <Border variant="ghost">
        <DropdownList
          items={actions.filter(a => !isEqual(a.state, issueState))}>
          {#snippet item(action)}
            <DropdownListItem
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
