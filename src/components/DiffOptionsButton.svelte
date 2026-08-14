<script lang="ts" module>
  import type { ComponentProps } from "svelte";

  import type { DiffOptions } from "@app/lib/diffOptions.svelte";

  import Icon from "@app/components/Icon.svelte";

  type DiffStyleOption = {
    value: DiffOptions["diffStyle"];
    icon: ComponentProps<typeof Icon>["name"];
    title: string;
  };

  const diffStyleOptions: DiffStyleOption[] = [
    { value: "unified", icon: "diff-unified", title: "Unified" },
    { value: "split", icon: "diff-split", title: "Split" },
  ];

  type IndicatorOption = {
    value: DiffOptions["indicators"];
    icon: ComponentProps<typeof Icon>["name"];
    title: string;
  };

  const indicatorOptions: IndicatorOption[] = [
    { value: "classic", icon: "diff-classic", title: "Classic (+/−)" },
    { value: "bars", icon: "diff-bars", title: "Bars" },
    { value: "none", icon: "eye-slash", title: "None" },
  ];

  type WordDiffOption = {
    value: DiffOptions["lineDiffType"];
    label: string;
    title: string;
  };

  const wordDiffOptions: WordDiffOption[] = [
    {
      value: "word-alt",
      label: "Word+",
      title: "Highlight entire words with enhanced algorithm",
    },
    {
      value: "word",
      label: "Word",
      title: "Highlight changed words within lines",
    },
    { value: "char", label: "Char", title: "Highlight character changes" },
    { value: "none", label: "None", title: "Show line-level changes only" },
  ];
</script>

<script lang="ts">
  import { diffOptions } from "@app/lib/diffOptions.svelte";

  import Button from "@app/components/Button.svelte";
  import Popover from "@app/components/Popover.svelte";

  let expanded = $state(false);
</script>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    /* Wide enough to keep every label on one line next to its buttons; the
       widest pairing is "Word diff" against its four options. */
    width: 23rem;
    padding: 0.75rem 1rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-md);
    background-color: var(--color-surface-canvas);
    font: var(--txt-body-m-regular);
    color: var(--color-text-primary);
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }
  .switch {
    display: flex;
    align-items: center;
  }
</style>

{#snippet boolSwitch(checked: boolean, onchange: (value: boolean) => void)}
  <div class="switch">
    <Button
      variant="ghost"
      flatRight
      active={checked}
      onclick={() => onchange(true)}>
      On
    </Button>
    <Button
      variant="ghost"
      flatLeft
      active={!checked}
      onclick={() => onchange(false)}>
      Off
    </Button>
  </div>
{/snippet}

<Popover placement="bottom-end" bind:expanded>
  {#snippet toggle(onclick)}
    <Button
      variant="naked"
      active={expanded}
      {onclick}
      title="Diff settings"
      styleHeight="2rem"
      styleWidth="2rem"
      stylePadding="0"
      styleJustifyContent="center">
      <Icon name="settings" />
    </Button>
  {/snippet}

  {#snippet popover()}
    <div class="panel">
      <div class="row">
        Diff style
        <div class="switch">
          {#each diffStyleOptions as option, index}
            <Button
              variant="ghost"
              flatLeft={index > 0}
              flatRight={index < diffStyleOptions.length - 1}
              active={diffOptions.diffStyle === option.value}
              title={option.title}
              onclick={() => (diffOptions.diffStyle = option.value)}
              styleJustifyContent="center">
              <Icon name={option.icon} />
            </Button>
          {/each}
        </div>
      </div>
      <div class="row">
        Word wrap
        {@render boolSwitch(
          diffOptions.wordWrap,
          value => (diffOptions.wordWrap = value),
        )}
      </div>
      <div class="row">
        Indicator style
        <div class="switch">
          {#each indicatorOptions as option, index}
            <Button
              variant="ghost"
              flatLeft={index > 0}
              flatRight={index < indicatorOptions.length - 1}
              active={diffOptions.indicators === option.value}
              title={option.title}
              onclick={() => (diffOptions.indicators = option.value)}
              styleJustifyContent="center">
              <Icon name={option.icon} />
            </Button>
          {/each}
        </div>
      </div>
      <div class="row">
        Word diff
        <div class="switch">
          {#each wordDiffOptions as option, index}
            <Button
              variant="ghost"
              flatLeft={index > 0}
              flatRight={index < wordDiffOptions.length - 1}
              active={diffOptions.lineDiffType === option.value}
              title={option.title}
              onclick={() => (diffOptions.lineDiffType = option.value)}>
              {option.label}
            </Button>
          {/each}
        </div>
      </div>
    </div>
  {/snippet}
</Popover>
