<script lang="ts">
  import type { Commit } from "@bindings/repo/Commit";
  import type { Snippet } from "svelte";

  import * as utils from "@app/lib/utils";

  import HoverPopover from "@app/components/HoverPopover.svelte";

  interface Props {
    children: Snippet;
    commit: Commit;
  }

  const { children, commit }: Props = $props();
</script>

<style>
  .authorship {
    display: flex;
    font: var(--txt-body-m-regular);
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
    font: var(--txt-code-regular);
  }
  .label {
    color: var(--color-text-secondary);
  }
  .avatar {
    width: 1rem;
    height: 1rem;
  }
</style>

<div class="authorship">
  <HoverPopover placement="top-start">
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
