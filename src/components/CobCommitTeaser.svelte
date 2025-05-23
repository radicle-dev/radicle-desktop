<script lang="ts">
  import type { Commit } from "@bindings/repo/Commit";

  import { twemoji } from "@app/lib/utils";

  import CompactCommitAuthorship from "@app/components/CompactCommitAuthorship.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import InlineTitle from "@app/components/InlineTitle.svelte";
  import NakedButton from "@app/components/NakedButton.svelte";

  interface Props {
    disabled: boolean;
    commit: Commit;
    onclick: () => void;
    hoverable?: boolean;
  }

  const { disabled, hoverable = false, commit, onclick }: Props = $props();

  let commitMessageVisible = $state(false);
</script>

<style>
  .teaser {
    display: flex;
    font-size: var(--font-size-small);
    align-items: start;
    padding: 0.125rem 0;
  }
  .message {
    align-items: center;
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    -webkit-touch-callout: initial;
    -webkit-user-select: text;
    user-select: text;
  }
  .left {
    display: flex;
    gap: 0.5rem;
    padding: 0 0.5rem;
    flex-direction: column;
  }
  .right {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-left: auto;
    height: 1.3125rem;
  }
  .commit-message {
    font-size: var(--font-size-tiny);
    -webkit-touch-callout: initial;
    -webkit-user-select: text;
    user-select: text;
  }
  .commit-expand-button {
    height: 1.3125rem;
    display: flex;
    align-items: center;
  }
  .disabled {
    color: var(--color-foreground-disabled) !important;
  }
  pre {
    white-space: pre-wrap;
    word-wrap: break-word;
  }
</style>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div class="teaser" class:disabled {onclick} aria-label="commit-teaser">
  <div class="left">
    <div class="message">
      <div style:cursor={hoverable ? "pointer" : "default"} use:twemoji>
        <InlineTitle fontSize="small" content={commit.summary} />
      </div>
      {#if commit.message.trim() !== commit.summary.trim()}
        <div class="commit-expand-button">
          <NakedButton
            stylePadding="0 0.25rem"
            variant="ghost"
            onclick={e => {
              e.stopPropagation();
              commitMessageVisible = !commitMessageVisible;
            }}>
            <Icon name="ellipsis" />
          </NakedButton>
        </div>
      {/if}
    </div>
    {#if commitMessageVisible}
      <div class="commit-message">
        <pre>{commit.message.replace(commit.summary, "").trim()}</pre>
      </div>
    {/if}
  </div>
  <div class="right">
    <CompactCommitAuthorship {commit}>
      <Id id={commit.id} variant={disabled ? "none" : "commit"} />
    </CompactCommitAuthorship>
  </div>
</div>
