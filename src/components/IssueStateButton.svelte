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

  export let state: State;
  export let save: (state: State) => Promise<void>;

  const actions: { caption: string; state: State }[] = [
    { caption: "Reopen", state: { status: "open" } },
    {
      caption: "Close as solved",
      state: { status: "closed", reason: "solved" },
    },
    { caption: "Close as other", state: { status: "closed", reason: "other" } },
  ];

  // Pick a default for the action button when the issue state changes.
  $: selectedAction = state.status === "open" ? actions[1] : actions[0];
</script>

<style>
  .main {
    display: flex;
    flex-direction: row;
    justify-content: center;
    gap: 1px;
  }
</style>

<div class="main">
  <Button
    variant="secondary"
    onclick={() => void save(selectedAction["state"])}>
    {selectedAction["caption"]}
  </Button>

  <Popover
    popoverPadding="0"
    popoverPositionTop="3rem"
    popoverPositionRight="0">
    <Button slot="toggle" let:toggle onclick={toggle} variant="secondary">
      <div style:height="22px" class="global-flex">
        <Icon name="chevron-down" />
      </div>
    </Button>
    <Border variant="ghost" slot="popover">
      <DropdownList items={actions.filter(a => !isEqual(a.state, state))}>
        <svelte:fragment slot="item" let:item={action}>
          <DropdownListItem
            selected={isEqual(selectedAction, action)}
            on:click={() => {
              selectedAction = action;
              closeFocused();
            }}>
            {action.caption}
          </DropdownListItem>
        </svelte:fragment>
      </DropdownList>
    </Border>
  </Popover>
</div>
