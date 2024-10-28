<script lang="ts">
  import type { Embed } from "@bindings/cob/thread/Embed";

  import ExtendedTextarea from "@app/components/ExtendedTextarea.svelte";
  import Border from "./Border.svelte";

  export let rid: string;
  export let placeholder: string | undefined = undefined;
  export let focus: boolean = false;
  export let submit: (comment: string, embeds: Embed[]) => Promise<void>;
  export let onclose: (() => void) | undefined = undefined;
  export let onexpand: (() => void) | undefined = undefined;
  export let disallowEmptyBody: boolean = false;

  let state: "collapsed" | "expanded" | "submit";

  $: state = onclose !== undefined ? "expanded" : "collapsed";
</script>

<style>
  .inactive {
    padding: 0 0.75rem;
    font-size: var(--font-size-small);
    color: var(--color-fill-gray);
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
    on:close={() => {
      if (onclose !== undefined) {
        onclose();
      } else {
        state = "collapsed";
      }
    }}
    on:submit={async ({ detail: { comment, embeds } }) => {
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
    onclick={() => {
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
