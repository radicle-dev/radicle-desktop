<script lang="ts">
  import { hide, modalStore } from "@app/lib/modal";
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
      onclick={$modalStore.disableScrimClose ? undefined : hide}
      style:cursor={$modalStore.disableHide ? "not-allowed" : "default"}>
    </div>
    <div class="content">
      <svelte:component this={$modalStore.component} {...$modalStore.props} />
    </div>
  </div>
{/if}
