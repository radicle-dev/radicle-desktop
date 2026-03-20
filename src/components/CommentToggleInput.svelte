<script lang="ts">
  import type { Embed } from "@bindings/cob/thread/Embed";

  import ExtendedTextarea from "@app/components/ExtendedTextarea.svelte";

  interface Props {
    rid: string;
    placeholder?: string;
    focus?: boolean;
    submit: (comment: string, embeds: Embed[]) => Promise<void>;
    onclose?: () => void;
    onexpand?: () => void;
    disallowEmptyBody?: boolean;
    // See `ExtendedTextarea`
    disableAttachments?: boolean | string;
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
    disableAttachments,
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
    font: var(--txt-body-m-regular);
    color: var(--color-text-quaternary);
  }
  .hoverable {
    background-color: var(--color-surface-base);
  }
  .hoverable:hover {
    background-color: var(--color-surface-canvas);
  }
</style>

{#if state !== "collapsed"}
  <ExtendedTextarea
    {disallowEmptyBody}
    {rid}
    {placeholder}
    borderVariant="ghost"
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
    {disableAttachments}
    submit={async ({ comment, embeds }) => {
      try {
        state = "submit";
        await submit(comment, Array.from(embeds.values()));
      } finally {
        state = "collapsed";
      }
    }} />
{:else}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div
    class="hoverable"
    style:border="1px solid var(--color-border-subtle)"
    style:border-radius="var(--border-radius-sm)"
    style:display="flex"
    style:gap="0.5rem"
    style:align-items="center"
    style:cursor="text"
    style:height="2rem"
    style:width="100%"
    role="button"
    tabindex="0"
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
  </div>
{/if}
