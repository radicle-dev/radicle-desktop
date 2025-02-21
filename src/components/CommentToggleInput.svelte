<script lang="ts">
  import type { Embed } from "@bindings/cob/thread/Embed";

  import ExtendedTextarea from "@app/components/ExtendedTextarea.svelte";
  import Border from "./Border.svelte";

  interface Props {
    rid: string;
    placeholder?: string;
    focus?: boolean;
    submit: (comment: string, embeds: Embed[]) => Promise<void>;
    onclose?: () => void;
    onexpand?: () => void;
    disallowEmptyBody?: boolean;
  }

  /* eslint-disable prefer-const */
  let {
    rid,
    placeholder,
    focus = false,
    submit,
    onclose,
    onexpand,
    disallowEmptyBody = false,
  }: Props = $props();
  /* eslint-enable prefer-const */

  let state: "collapsed" | "expanded" | "submit" | undefined = $state();

  $effect(() => {
    state = onclose !== undefined ? "expanded" : "collapsed";
  });
</script>

<style>
  .inactive {
    padding: 0 0.75rem;
    font-size: var(--font-size-small);
    color: var(--color-fill-gray);
    font-family: var(--font-family-sans-serif);
  }
</style>

{#if state !== "collapsed"}
  <ExtendedTextarea
    {disallowEmptyBody}
    {rid}
    {placeholder}
    submitInProgress={state === "submit"}
    {focus}
    stylePadding="0.5rem 0.75rem"
    close={() => {
      if (onclose !== undefined) {
        onclose();
      } else {
        state = "collapsed";
      }
    }}
    submit={async ({ comment, embeds }) => {
      try {
        state = "submit";
        await submit(comment, Array.from(embeds.values()));
      } finally {
        state = "collapsed";
      }
    }} />
{:else}
  <Border
    hoverable
    styleCursor="text"
    variant="float"
    styleHeight="40px"
    styleWidth="100%"
    onclick={e => {
      e.preventDefault();
      e.stopPropagation();

      state = "expanded";
      if (onexpand !== undefined) {
        onexpand();
      }
    }}>
    <div style:width="100%" class="inactive">
      {placeholder}
    </div>
  </Border>
{/if}
