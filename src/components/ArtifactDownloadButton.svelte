<script lang="ts">
  import type { Artifact } from "@bindings/cob/release/Artifact";

  import Button from "@app/components/Button.svelte";
  import Command from "@app/components/Command.svelte";
  import DelegateBadge from "@app/components/DelegateBadge.svelte";
  import ExternalLink from "@app/components/ExternalLink.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Popover from "@app/components/Popover.svelte";

  interface Props {
    artifact: Artifact;
    delegateIds: Set<string>;
    rid: string;
  }

  const { artifact, delegateIds, rid }: Props = $props();

  let activeTab: "cli" | "browser" = $state("cli");

  // Whether the location can be opened in a browser, as opposed to being
  // served over the radicle-artifact protocol.
  function isWebUrl(url: string): boolean {
    return /^https?:\/\//i.test(url);
  }

  // Delegate locations come first: they are the ones a reader can trust most.
  const webLocations = $derived(
    [...artifact.locations.filter(l => isWebUrl(l.url))].sort(
      (a, b) =>
        Number(delegateIds.has(b.user.did)) -
        Number(delegateIds.has(a.user.did)),
    ),
  );
  // Any location at all can be fetched with the CLI, web locations included.
  const downloadable = $derived(artifact.locations.length > 0);
  const command = $derived(
    `rad artifact -r ${rid} download --cid ${artifact.cid}`,
  );
</script>

<style>
  .popover {
    font: var(--txt-body-m-regular);
    width: 24rem;
  }
  .tabs {
    display: flex;
    gap: 2px;
    margin-bottom: 1rem;
  }
  label {
    display: block;
    margin-bottom: 0.75rem;
    color: var(--color-text-secondary);
  }
  .warning {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
    color: var(--color-feedback-warning-text);
  }
  .locations {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .location {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
  }
  .url {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--color-text-secondary);
  }
</style>

<Popover placement="bottom-end" popoverPadding="1rem">
  {#snippet toggle(onclick)}
    <Button
      {onclick}
      variant="secondary"
      styleHeight="1.75rem"
      disabled={!downloadable}
      title={downloadable
        ? undefined
        : "No download source is currently available for this artifact"}>
      <Icon name="download" />Download
    </Button>
  {/snippet}

  {#snippet popover()}
    <div class="popover">
      <div class="tabs">
        <Button
          styleWidth="100%"
          flatRight
          bordered
          active={activeTab === "cli"}
          onclick={() => (activeTab = "cli")}>
          <Icon name="logo" />CLI
        </Button>
        <Button
          styleWidth="100%"
          flatLeft
          bordered
          disabled={webLocations.length === 0}
          title={webLocations.length === 0
            ? "This artifact has no web locations"
            : undefined}
          active={activeTab === "browser"}
          onclick={() => (activeTab = "browser")}>
          <Icon name="download" />Browser
        </Button>
      </div>

      {#if activeTab === "cli"}
        <label for="download-command">
          Use the Radicle Artifact CLI to download and verify this artifact.
        </label>
        <Command {command} styleWidth="100%" />
      {:else}
        <div class="warning">
          <Icon name="warning" />
          These downloads are not verified.
        </div>
        <div class="locations">
          {#each webLocations as location (location.url)}
            <div class="location">
              {#if delegateIds.has(location.user.did)}
                <DelegateBadge tooltip="Location added by a delegate" />
              {/if}
              <span class="url" title={location.url}>{location.url}</span>
              <ExternalLink href={location.url} title="Download" />
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/snippet}
</Popover>
