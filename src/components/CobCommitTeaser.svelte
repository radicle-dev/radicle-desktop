<script lang="ts">
  import type { Commit } from "@bindings/repo/Commit";

  import { twemoji } from "@app/lib/utils";

  import CompactCommitAuthorship from "@app/components/CompactCommitAuthorship.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import InlineTitle from "@app/components/InlineTitle.svelte";
  import NakedButton from "@app/components/NakedButton.svelte";

  interface Props {
    commit: Commit;
  }

  const { commit }: Props = $props();

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
    height: 21px;
  }
  .commit-message {
    font-size: var(--font-size-tiny);
  }
  .commit-expand-button {
    height: 21px;
    display: flex;
    align-items: center;
  }
  pre {
    white-space: pre-wrap;
    word-wrap: break-word;
  }
</style>

<div class="teaser" aria-label="commit-teaser">
  <div class="left">
    <div class="message">
      <div class="summary" use:twemoji>
        <InlineTitle fontSize="small" content={commit.summary} />
      </div>
      {#if commit.message.trim() !== commit.summary.trim()}
        <div class="commit-expand-button">
          <NakedButton
            stylePadding="0 4px"
            variant="ghost"
            onclick={() => {
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
      <Id id={commit.id} variant="commit" />
    </CompactCommitAuthorship>
  </div>
</div>
