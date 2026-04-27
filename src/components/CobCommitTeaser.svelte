<script lang="ts">
  import type { Commit } from "@bindings/repo/Commit";
  import type { Snippet } from "svelte";

  import { twemoji } from "@app/lib/utils";

  import Button from "@app/components/Button.svelte";
  import CompactCommitAuthorship from "@app/components/CompactCommitAuthorship.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import InlineTitle from "@app/components/InlineTitle.svelte";

  interface Props {
    children?: Snippet;
    commit: Commit;
    disabled: boolean;
    hoverable?: boolean;
    onclick?: () => void;
    flush?: boolean;
  }

  const {
    children,
    commit,
    disabled,
    flush = false,
    hoverable = false,
    onclick = undefined,
  }: Props = $props();

  let commitMessageVisible = $state(false);
</script>

<style>
  .teaser {
    display: flex;
    font: var(--txt-body-m-regular);
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
    font: var(--txt-body-s-regular);
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
    color: var(--color-text-disabled) !important;
  }
  pre {
    white-space: pre-wrap;
    word-wrap: break-word;
  }
</style>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div class="teaser" class:disabled {onclick} aria-label="commit-teaser">
  <div class="left" style:padding={flush ? "0" : undefined}>
    <div class="message">
      <div style:cursor={hoverable ? "pointer" : "default"} use:twemoji>
        <InlineTitle fontSize="small" content={commit.summary} />
      </div>
      {#if commit.message.trim() !== commit.summary.trim()}
        <div class="commit-expand-button">
          <Button
            variant="naked"
            stylePadding="0 0.25rem"
            onclick={e => {
              e.stopPropagation();
              commitMessageVisible = !commitMessageVisible;
            }}>
            <Icon name="ellipsis" />
          </Button>
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
    {@render children?.()}
    <CompactCommitAuthorship {commit}>
      <Id id={commit.id} clipboard={commit.id} />
    </CompactCommitAuthorship>
  </div>
</div>
