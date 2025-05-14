<script lang="ts">
  import type { Review } from "@bindings/cob/patch/Review";

  import capitalize from "lodash/capitalize.js";

  import { closeFocused } from "./Popover.svelte";
  import { verdictIcon } from "@app/lib/utils";

  import Border from "@app/components/Border.svelte";
  import DropdownList from "@app/components/DropdownList.svelte";
  import DropdownListItem from "@app/components/DropdownListItem.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Popover from "@app/components/Popover.svelte";
  import VerdictBadge from "@app/components/VerdictBadge.svelte";

  interface Props {
    onSelect: (selectedVerdict: Review["verdict"]) => Promise<void>;
    summaryMissing: boolean;
    selectedVerdict: Review["verdict"];
  }

  const { onSelect, summaryMissing, selectedVerdict }: Props = $props();

  let popoverExpanded: boolean = $state(false);
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
  .accepted {
    color: var(--color-foreground-success);
  }

  .rejected {
    color: var(--color-foreground-red);
  }

  .no-verdict {
    color: var(--color-foreground-dim);
  }
</style>

<Popover
  popoverPadding="0"
  popoverPositionLeft="0"
  popoverPositionTop="2rem"
  bind:expanded={popoverExpanded}>
  {#snippet toggle(onclick)}
    <button {onclick}>
      <VerdictBadge verdict={selectedVerdict} hoverable>
        <Icon name={popoverExpanded ? "chevron-up" : "chevron-down"} />
      </VerdictBadge>
    </button>
  {/snippet}
  {#snippet popover()}
    <Border variant="ghost">
      <DropdownList items={[undefined, "accept", "reject"] as const}>
        {#snippet item(verdict)}
          <DropdownListItem
            title={verdict === undefined && summaryMissing
              ? "Set a summary to select verdict None"
              : undefined}
            disabled={verdict === undefined && summaryMissing}
            selected={selectedVerdict === verdict}
            onclick={async () => {
              await onSelect(verdict);
              closeFocused();
            }}>
            <span
              class="global-flex"
              class:accepted={verdict === "accept"}
              class:rejected={verdict === "reject"}
              class:no-verdict={verdict === undefined}>
              <Icon name={verdictIcon(verdict)} />
              {verdict ? capitalize(`${verdict}ed`) : "None"}
            </span>
          </DropdownListItem>
        {/snippet}
      </DropdownList>
    </Border>
  {/snippet}
</Popover>
