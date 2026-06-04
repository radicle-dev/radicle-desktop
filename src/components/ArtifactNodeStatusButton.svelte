<script lang="ts">
  import { artifactNodeRunning } from "@app/lib/events";

  import Button from "@app/components/Button.svelte";
  import Command from "@app/components/Command.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Popover from "@app/components/Popover.svelte";

  let popoverExpanded: boolean = $state(false);
</script>

<Popover
  popoverPadding="0"
  placement="top-start"
  bind:expanded={popoverExpanded}>
  {#snippet toggle(onclick)}
    <Button
      variant="naked"
      {onclick}
      active={popoverExpanded}
      styleWidth="100%"
      styleJustifyContent="flex-start">
      {#if $artifactNodeRunning}
        <span style:color="var(--color-text-tertiary)">
          <Icon name="online" />
        </span>
        Artifacts online
      {:else}
        <span style:color="var(--color-text-tertiary)">
          <Icon name="offline" />
        </span>
        Artifacts offline
      {/if}
    </Button>
  {/snippet}
  {#snippet popover()}
    <div
      style:border="1px solid var(--color-border-subtle)"
      style:border-radius="var(--border-radius-sm)"
      style:display="flex"
      style:gap="0.5rem"
      style:align-items="flex-start"
      style:background-color="var(--color-surface-canvas)"
      style:padding="1rem"
      style:width="22rem"
      style:flex-direction="column">
      <div class="txt-body-m-regular" style:line-height="1.625rem">
        {#if $artifactNodeRunning}
          The artifact node is running; you can seed and download release
          artifacts.
        {:else}
          The artifact node is not running. Seeding and downloading artifacts
          are unavailable until you start it.

          <div style:margin-top="1rem">
            Install <code>rad-artifact</code>
            and start the node with:
            <div style:margin-top="0.5rem">
              <Command
                styleWidth="fit-content"
                command="rad-artifact node start" />
            </div>
          </div>
        {/if}
      </div>
    </div>
  {/snippet}
</Popover>
