<script lang="ts">
  import type { Commit } from "@bindings/repo/Commit";
  import type { Snippet } from "svelte";

  import * as utils from "@app/lib/utils";
  import HoverPopover from "./HoverPopover.svelte";

  interface Props {
    children: Snippet;
    commit: Commit;
  }

  const { children, commit }: Props = $props();
</script>

<style>
  .authorship {
    display: flex;
    font-size: var(--font-size-small);
    column-gap: 0.5rem;
    align-items: center;
    white-space: nowrap;
  }
  .person {
    display: flex;
    align-items: center;
    flex-wrap: nowrap;
    white-space: nowrap;
    gap: 0.5rem;
    font-family: var(--font-family-monospace);
    font-weight: var(--font-weight-semibold);
  }
  .label {
    font-family: var(--font-family-sans-serif);
    font-weight: var(--font-weight-regular);
    color: var(--color-foreground-dim);
  }
  .avatar {
    width: 1rem;
    height: 1rem;
    clip-path: var(--1px-corner-fill);
  }
</style>

<div class="authorship">
  <HoverPopover
    stylePopoverPositionLeft="-8rem"
    stylePopoverPositionBottom="1.5rem">
    {#snippet toggle()}
      <div style="display: flex;">
        {#if commit.author.email === commit.committer.email}
          <img
            class="avatar"
            alt="avatar"
            src={utils.gravatarURL(commit.committer.email)} />
        {:else}
          <img
            style:margin-right="0.25rem"
            class="avatar"
            alt="avatar"
            src={utils.gravatarURL(commit.author.email)} />
          <img
            class="avatar"
            alt="avatar"
            src={utils.gravatarURL(commit.committer.email)} />
        {/if}
      </div>
    {/snippet}

    {#snippet popover()}
      <div class="popover">
        {#if commit.author.email === commit.committer.email}
          <div class="person">
            <div class="label">Author</div>
            <img
              class="avatar"
              alt="avatar"
              src={utils.gravatarURL(commit.committer.email)} />
            {commit.author.name}
          </div>
        {:else}
          <div class="person">
            <div class="label">Author</div>
            <img
              class="avatar"
              alt="avatar"
              src={utils.gravatarURL(commit.author.email)} />
            {commit.author.name}
          </div>
          <div class="person">
            <div class="label">Committer</div>
            <img
              class="avatar"
              alt="avatar"
              src={utils.gravatarURL(commit.committer.email)} />
            {commit.committer.name}
          </div>
        {/if}
      </div>
    {/snippet}
  </HoverPopover>
  {@render children()}
  <div title={utils.absoluteTimestamp(commit.committer.time * 1000)}>
    {utils.formatTimestamp(commit.committer.time * 1000)}
  </div>
</div>
