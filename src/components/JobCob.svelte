<script lang="ts">
  import type { Job } from "@bindings/repo/Job";

  import { invoke } from "@app/lib/invoke";
  import { authorForNodeId } from "@app/lib/utils";

  import HoverPopover from "@app/components/HoverPopover.svelte";
  import Icon from "@app/components/Icon.svelte";

  import DropdownListItem from "./DropdownListItem.svelte";
  import NodeId from "./NodeId.svelte";

  interface Props {
    commit: string;
    rid: string;
  }

  const { commit, rid }: Props = $props();
</script>

<style>
  .status {
    gap: 0.5rem;
    width: fit-content;
  }
</style>

{#await invoke<Job[]>("list_jobs", { rid, sha: commit }) then jobs}
  {#if jobs.length > 0}
    <HoverPopover
      stylePadding="0.25rem"
      stylePopoverPositionBottom="2rem"
      stylePopoverPositionRight="-1.5rem">
      {#snippet toggle()}
        {#if jobs.every(j => {
          return j.runs.every(r => {
            return r.status === "succeeded";
          });
        })}
          <div
            class="global-counter"
            style:padding="0"
            style:color="var(--color-fill-success)"
            style:background-color="var(--color-fill-diff-green)">
            <Icon name="checkmark-double" />
          </div>
        {:else if jobs.every(j => {
          return j.runs.every(r => {
            return r.status === "failed";
          });
        })}
          <div
            class="global-counter"
            style:color="var(--color-foreground-red)"
            style:padding="0"
            style:background-color="var(--color-fill-diff-red)">
            <Icon name="cross-double" />
          </div>
        {:else}
          <div
            class="global-counter"
            style:padding="0"
            style:color="var(--color-fill-gray)"
            style:background-color="var(--color-fill-ghost)">
            <Icon name="help" />
          </div>
        {/if}
      {/snippet}

      {#snippet popover()}
        {#each jobs as job}
          {#each job.runs as run}
            <a
              style:text-decoration="none"
              style:cursor="pointer"
              style:width="100%"
              href={run.log}
              target="_blank">
              <DropdownListItem styleGap="0.5rem" selected={true}>
                {#if run.status === "started"}
                  <div
                    class="global-counter status"
                    style:background-color="var(--color-fill-float)"
                    style:color="var(--color-fill-gray)">
                    <Icon name="hourglass" /> Started
                  </div>
                {:else if run.status === "failed"}
                  <div
                    class="global-counter status"
                    style:color="var(--color-foreground-red)"
                    style:background-color="var(--color-fill-diff-red)">
                    <Icon name="cross" /> Failed
                  </div>
                {:else if run.status === "succeeded"}
                  <div
                    class="global-counter status"
                    style:color="var(--color-fill-success)"
                    style:background-color="var(--color-fill-diff-green)">
                    <Icon name="checkmark" /> Passed
                  </div>
                {/if}
                <NodeId {...authorForNodeId(run.node)} />
                <div
                  style:margin-left="auto"
                  style:color="var(--color-fill-gray)">
                  <Icon name="open-external" />
                </div>
              </DropdownListItem>
            </a>
          {/each}
        {/each}
      {/snippet}
    </HoverPopover>
  {/if}
{/await}
