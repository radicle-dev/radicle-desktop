<script lang="ts">
  import type { Revision } from "@bindings/cob/patch/Revision";

  interface Props {
    revision: Revision;
    revisions: Revision[];
  }

  /* eslint-disable prefer-const */
  let { revision, revisions }: Props = $props();
  /* eslint-enable prefer-const */
</script>

{#if revision.id === revisions.slice(-1)[0].id}
  <span
    class="global-counter"
    style:height="1.375rem"
    style:color="var(--color-foreground-contrast)">
    Latest
  </span>
{/if}
{#if revision.id === revisions[0].id}
  <span
    class="global-counter"
    style:height="1.375rem"
    style:color="var(--color-foreground-contrast)">
    Initial
  </span>
{/if}
