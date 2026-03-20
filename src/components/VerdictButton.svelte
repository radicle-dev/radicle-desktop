<script lang="ts">
  import type { Review } from "@bindings/cob/patch/Review";

  import capitalize from "lodash/capitalize.js";

  import { verdictIcon } from "@app/lib/utils";

  import Button from "@app/components/Button.svelte";
  import DropdownList from "@app/components/DropdownList.svelte";
  import DropdownListItem from "@app/components/DropdownListItem.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Popover from "@app/components/Popover.svelte";
  import { closeFocused } from "@app/components/Popover.svelte";

  interface Props {
    onSelect: (selectedVerdict: Review["verdict"]) => Promise<void>;
    draft: boolean;
    summaryMissing: boolean;
    selectedVerdict: Review["verdict"];
  }

  const { onSelect, draft, summaryMissing, selectedVerdict }: Props = $props();

  let popoverExpanded: boolean = $state(false);

  function verdictBgColor(verdict: Review["verdict"]): string {
    if (verdict === "accept") return "var(--color-feedback-success-bg)";
    if (verdict === "reject") return "var(--color-feedback-error-bg)";
    return "var(--color-surface-subtle)";
  }

  function verdictColor(verdict: Review["verdict"]): string {
    if (verdict === "accept") return "var(--color-feedback-success-text)";
    if (verdict === "reject") return "var(--color-feedback-error-text)";
    return "var(--color-text-secondary)";
  }
</script>

<Popover
  popoverPadding="0"
  placement="bottom-start"
  bind:expanded={popoverExpanded}>
  {#snippet toggle(onclick)}
    <Button variant="outline" {onclick} active={popoverExpanded}>
      <span
        class="global-chip"
        style:padding="0"
        style:margin-left="-0.25rem"
        style:background-color={verdictBgColor(selectedVerdict)}
        style:color={verdictColor(selectedVerdict)}>
        <Icon name={verdictIcon(selectedVerdict)} />
      </span>
      <span style:color="var(--color-text-secondary)">
        {selectedVerdict ? capitalize(`${selectedVerdict}ed`) : "None"}
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
      <DropdownList items={[undefined, "accept", "reject"] as const}>
        {#snippet item(verdict)}
          <DropdownListItem
            title={verdict === undefined && summaryMissing
              ? "Set a summary to select verdict None"
              : undefined}
            disabled={draft === false &&
              verdict === undefined &&
              summaryMissing}
            selected={selectedVerdict === verdict}
            styleGap="0.5rem"
            onclick={async () => {
              await onSelect(verdict);
              closeFocused();
            }}>
            <span
              class="global-chip"
              style:padding="0"
              style:margin-left="-0.5rem"
              style:background-color={verdictBgColor(verdict)}
              style:color={verdictColor(verdict)}>
              <Icon name={verdictIcon(verdict)} />
            </span>
            <span style:color="var(--color-text-secondary)">
              {verdict ? capitalize(`${verdict}ed`) : "None"}
            </span>
          </DropdownListItem>
        {/snippet}
      </DropdownList>
    </div>
  {/snippet}
</Popover>
