<script lang="ts">
  import { cubicOut } from "svelte/easing";
  import { fade } from "svelte/transition";

  import { hide, modalStore } from "@app/lib/modal";

  // Rise up and scale in on enter; fall back down on exit.
  function riseFall(_node: Element, { duration = 200 } = {}) {
    return {
      duration,
      easing: cubicOut,
      css: (t: number, u: number) =>
        `opacity: ${t}; transform: translateY(${u * 16}px) scale(${0.96 + 0.04 * t});`,
    };
  }
</script>

<style>
  .container {
    height: 100vh;
    width: 100vw;
    position: fixed;
    z-index: 300;
    justify-content: center;
    overflow: scroll;
    display: flex;
  }

  .overlay {
    background-color: var(--color-surface-scrim);
    height: 100%;
    width: 100%;
    position: fixed;
  }

  .content {
    z-index: 200;
    margin: auto;
  }
</style>

{#if $modalStore}
  <div class="container">
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
      role="button"
      tabindex="0"
      class="overlay"
      transition:fade={{ duration: 150 }}
      onclick={$modalStore.disableScrimClose ? undefined : hide}
      style:cursor={$modalStore.disableHide ? "not-allowed" : "default"}>
    </div>
    <div class="content" transition:riseFall>
      <svelte:component this={$modalStore.component} {...$modalStore.props} />
    </div>
  </div>
{/if}
