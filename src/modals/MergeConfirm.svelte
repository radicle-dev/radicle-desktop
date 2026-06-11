<script lang="ts">
  import { hide } from "@app/lib/modal";

  import Button from "@app/components/Button.svelte";
  import Icon from "@app/components/Icon.svelte";

  interface Props {
    source: string;
    branch: string;
    commit: string;
    onConfirm: () => void;
  }

  const { source, branch, commit, onConfirm }: Props = $props();

  // Haphazardly scattered merge icons decorating the header.
  const decorIcons = [
    { top: "-10%", left: "4%", rotate: -18, scale: 2, opacity: 0.16 },
    { top: "52%", left: "13%", rotate: 14, scale: 1.1, opacity: 0.1 },
    { top: "8%", left: "27%", rotate: 30, scale: 1.4, opacity: 0.12 },
    { top: "60%", left: "40%", rotate: -12, scale: 0.9, opacity: 0.1 },
    { top: "-4%", left: "55%", rotate: 22, scale: 1.6, opacity: 0.14 },
    { top: "46%", left: "68%", rotate: -26, scale: 1.2, opacity: 0.12 },
    { top: "2%", left: "84%", rotate: 10, scale: 2.1, opacity: 0.16 },
    { top: "58%", left: "92%", rotate: -8, scale: 1, opacity: 0.1 },
  ];
</script>

<style>
  .modal {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    width: 28rem;
    max-width: 90vw;
    padding: 1.5rem;
    background-color: var(--color-surface-canvas);
    border-radius: var(--border-radius-md);
    box-shadow: var(--elevation-medium);
  }
  .header {
    position: relative;
    margin: -1.5rem -1.5rem 0;
    padding: 1.5rem 1.5rem 0;
    overflow: hidden;
    border-top-left-radius: var(--border-radius-md);
    border-top-right-radius: var(--border-radius-md);
    background: radial-gradient(
      120% 140% at 0% 0%,
      var(--color-surface-merged),
      transparent 55%
    );
  }
  .decoration {
    position: absolute;
    inset: 0;
    pointer-events: none;
  }
  .scatter {
    position: absolute;
    display: inline-flex;
    color: var(--color-text-merged);
  }
  .title {
    position: relative;
    z-index: 1;
    font: var(--txt-heading-m);
    color: var(--color-text-primary);
  }
  .branches {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: var(--color-text-secondary);
  }
  .branches .mono {
    padding: 0.0625rem 0.375rem;
    border-radius: var(--border-radius-sm);
    background-color: var(--color-surface-subtle);
  }
  .body {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    color: var(--color-text-secondary);
  }
  .body p {
    margin: 0;
  }
  .mono {
    font: var(--txt-code-regular);
    color: var(--color-text-primary);
  }
  .caption {
    color: var(--color-text-tertiary);
  }
  .actions {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 0.5rem;
  }
</style>

<div class="modal">
  <div class="header">
    <div class="decoration" aria-hidden="true">
      {#each decorIcons as ic, i (i)}
        <span
          class="scatter"
          style:top={ic.top}
          style:left={ic.left}
          style:opacity={ic.opacity}
          style:transform="rotate({ic.rotate}deg) scale({ic.scale})">
          <Icon name="patch-merged" />
        </span>
      {/each}
    </div>
    <div class="title">Merge patch</div>
  </div>
  <div class="branches txt-body-s-regular">
    <span class="mono">{source}</span>
    <Icon name="arrow-right" />
    <span class="mono">{branch}</span>
  </div>
  <div class="body txt-body-m-regular">
    <p>
      This records the patch as merged and fast-forwards the repository's
      canonical <strong>{branch}</strong>
      branch to commit
      <span class="mono">{commit.slice(0, 7)}</span>
      .
    </p>
    <p class="caption">
      The canonical branch is the shared source of truth for everyone. Your
      local checkout isn't updated automatically, so fetch the repo to catch up.
    </p>
  </div>
  <div class="actions">
    <Button variant="naked" onclick={() => hide()}>Cancel</Button>
    <Button
      variant="secondary"
      onclick={() => {
        onConfirm();
        hide();
      }}>
      Merge patch
    </Button>
  </div>
</div>
