<script lang="ts">
  import type { Review } from "@bindings/cob/patch/Review";

  import capitalize from "lodash/capitalize.js";

  import { closeFocused } from "./Popover.svelte";
  import { verdictIcon, verdictIconColor } from "@app/lib/utils";

  import Border from "@app/components/Border.svelte";
  import DropdownList from "@app/components/DropdownList.svelte";
  import DropdownListItem from "@app/components/DropdownListItem.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Popover from "@app/components/Popover.svelte";
  import VerdictBadge from "@app/components/VerdictBadge.svelte";

  interface Props {
    onSelect: (verdict: Review["verdict"]) => Promise<void>;
    summaryMissing: boolean;
    verdict: Review["verdict"];
  }

  const { onSelect, summaryMissing, verdict }: Props = $props();
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
</style>

<Popover popoverPadding="0" popoverPositionLeft="0" popoverPositionTop="2rem">
  {#snippet toggle(onclick)}
    <button {onclick}>
      <VerdictBadge {verdict} hoverable>
        <Icon name="chevron-down" />
      </VerdictBadge>
    </button>
  {/snippet}
  {#snippet popover()}
    <Border variant="ghost">
      <DropdownList items={[undefined, "accept", "reject"] as const}>
        {#snippet item(action)}
          <DropdownListItem
            title={action === undefined && summaryMissing
              ? "Set a summary to select verdict None"
              : undefined}
            disabled={action === undefined && summaryMissing}
            selected={verdict === action}
            onclick={async () => {
              await onSelect(action);
              closeFocused();
            }}>
            <span class="global-flex" style:color={verdictIconColor(action)}>
              <Icon name={verdictIcon(action)} />
              {action ? capitalize(`${action}ed`) : "None"}
            </span>
          </DropdownListItem>
        {/snippet}
      </DropdownList>
    </Border>
  {/snippet}
</Popover>
