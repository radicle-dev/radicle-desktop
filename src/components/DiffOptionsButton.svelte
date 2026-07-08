<script lang="ts" module>
  import type { ComponentProps } from "svelte";

  import type { DiffOptions } from "@app/lib/diffOptions.svelte";

  import Icon from "@app/components/Icon.svelte";

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
    width: 20rem;
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
      styleHeight="1.75rem"
      styleWidth="1.75rem"
      stylePadding="0"
      styleJustifyContent="center">
      <Icon name="settings" />
    </Button>
  {/snippet}

  {#snippet popover()}
    <div class="panel">
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
