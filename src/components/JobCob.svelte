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
    <HoverPopover stylePadding="0.25rem" placement="bottom-end">
      {#snippet toggle()}
        {#if jobs.every(j => {
          return j.runs.every(r => {
            return r.status === "succeeded";
          });
        })}
          <div
            class="global-chip"
            style:padding="0"
            style:color="var(--color-feedback-success-text)"
            style:background-color="var(--color-feedback-success-bg)">
            <Icon name="checkmark" />
          </div>
        {:else if jobs.every(j => {
          return j.runs.every(r => {
            return r.status === "failed";
          });
        })}
          <div
            class="global-chip"
            style:color="var(--color-feedback-error-text)"
            style:padding="0"
            style:background-color="var(--color-feedback-error-bg)">
            <Icon name="close" />
          </div>
        {:else}
          <div
            class="global-chip"
            style:padding="0"
            style:color="var(--color-text-quaternary)"
            style:background-color="var(--color-surface-subtle)">
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
                    class="global-chip status"
                    style:background-color="var(--color-surface-canvas)"
                    style:color="var(--color-text-quaternary)">
                    <Icon name="hourglass" /> Started
                  </div>
                {:else if run.status === "failed"}
                  <div
                    class="global-chip status"
                    style:color="var(--color-feedback-error-text)"
                    style:background-color="var(--color-feedback-error-bg)">
                    <Icon name="close" /> Failed
                  </div>
                {:else if run.status === "succeeded"}
                  <div
                    class="global-chip status"
                    style:color="var(--color-feedback-success-text)"
                    style:background-color="var(--color-feedback-success-bg)">
                    <Icon name="checkmark" /> Passed
                  </div>
                {/if}
                <NodeId {...authorForNodeId(run.node)} />
                <div
                  style:margin-left="auto"
                  style:color="var(--color-text-quaternary)">
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
