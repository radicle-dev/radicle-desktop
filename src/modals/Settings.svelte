<script lang="ts">
  import { hide } from "@app/lib/modal";
  import { updateChecker } from "@app/lib/updateChecker.svelte";

  import AnnounceSwitch from "@app/components/AnnounceSwitch.svelte";
  import BadgeCounterSwitch from "@app/components/BadgeCounterSwitch.svelte";
  import Button from "@app/components/Button.svelte";
  import CodeFontSwitch from "@app/components/CodeFontSwitch.svelte";
  import ExternalLink from "@app/components/ExternalLink.svelte";
  import FontSizeSwitch from "@app/components/FontSizeSwitch.svelte";
  import Icon from "@app/components/Icon.svelte";
  import ThemeSwitch from "@app/components/ThemeSwitch.svelte";
  import UpdateSwitch from "@app/components/UpdateSwitch.svelte";
</script>

<style>
  .modal {
    width: 40rem;
    display: flex;
    flex-direction: column;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-lg);
    background-color: var(--color-surface-canvas);
    overflow: hidden;
  }
  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 1.5rem;
    height: 3.25rem;
    flex-shrink: 0;
    border-bottom: 1px solid var(--color-border-subtle);
  }
  .title {
    font: var(--txt-heading-s);
    color: var(--color-text-primary);
  }
  .rows {
    display: flex;
    flex-direction: column;
    padding: 1.5rem;
    gap: 1.5rem;
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }
  .row-label {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
    min-width: 0;
  }
  .row-title {
    font: var(--txt-body-m-medium);
    color: var(--color-text-primary);
  }
  .row-description {
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
  }
  .footer {
    padding: 7rem 1.5rem 1.5rem;
    font: var(--txt-body-m-regular);
    color: var(--color-text-tertiary);
  }
</style>

<div class="modal">
  <div class="header">
    <span class="title">Settings</span>
    <Button variant="naked" onclick={hide}>
      <span style:color="var(--color-text-tertiary)">
        <Icon name="close" />
      </span>
    </Button>
  </div>
  <div class="rows">
    <div class="row">
      <div class="row-label">
        <span class="row-title">Appearance</span>
        <span class="row-description">Light, dark, or follow your system</span>
      </div>
      <ThemeSwitch />
    </div>
    <div class="row">
      <div class="row-label">
        <span class="row-title">Font size</span>
        <span class="row-description">
          Make the interface text larger or smaller
        </span>
      </div>
      <FontSizeSwitch />
    </div>
    <div class="row">
      <div class="row-label">
        <span class="row-title">Notification badge</span>
        <span class="row-description">Show unread count on the dock icon</span>
      </div>
      <BadgeCounterSwitch />
    </div>
    <div class="row">
      <div class="row-label">
        <span class="row-title">Announce changes</span>
        <span class="row-description">
          Broadcast your activity to the network right away or periodically
        </span>
      </div>
      <AnnounceSwitch />
    </div>
    <div class="row">
      <div class="row-label">
        <span class="row-title">Code font</span>
        <span class="row-description">Use a monospace font in code views</span>
      </div>
      <CodeFontSwitch />
    </div>
    <div class="row">
      <div class="row-label">
        <span class="row-title">Notify on new versions</span>
        <span class="row-description">
          Check for new versions in the background
        </span>
      </div>
      <UpdateSwitch
        active={updateChecker.isEnabled}
        disable={updateChecker.disable}
        enable={updateChecker.enable} />
    </div>
  </div>
  <div class="footer">
    {#if updateChecker.currentVersion}
      <span class="txt-selectable">
        Version {updateChecker.currentVersion}
      </span>
      {#if updateChecker.newVersion}
        · <ExternalLink href="https://radicle.dev/desktop">
          Update to {updateChecker.newVersion}
        </ExternalLink>
      {:else}
        · Up to date
      {/if}
    {/if}
  </div>
</div>
