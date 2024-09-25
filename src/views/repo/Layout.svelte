<script lang="ts">
  import Header from "@app/components/Header.svelte";
  import Icon from "@app/components/Icon.svelte";
  import NakedButton from "@app/components/NakedButton.svelte";

  let hidden = false;
</script>

<style>
  .layout {
    display: grid;
    grid-template: auto 1fr auto / auto 1fr auto;
    height: 100%;
  }

  .header {
    grid-column: 1 / 4;
    border-bottom: 2px solid var(--color-background-default);
  }

  .sidebar {
    grid-column: 1 / 2;
    margin: 1rem 0.5rem 0 1rem;
    min-width: 14rem;
    overflow: scroll;
  }

  .content {
    padding-top: 1rem;
    grid-column: 2 / 3;
    overflow: scroll;
    overscroll-behavior: none;
  }

  .hidden {
    display: none;
  }
</style>

<div class="layout">
  <div class="header">
    <Header>
      <svelte:fragment slot="icon-left">
        <NakedButton
          variant="ghost"
          onclick={() => {
            hidden = !hidden;
          }}>
          <Icon name="sidebar" />
        </NakedButton>
      </svelte:fragment>

      <svelte:fragment slot="center">
        <slot name="header-center" />
      </svelte:fragment>

      <svelte:fragment slot="breadcrumbs">
        <slot name="breadcrumbs" />
      </svelte:fragment>
    </Header>
  </div>

  <div class="sidebar" class:hidden>
    <slot name="sidebar" />
  </div>

  <div class="content">
    <slot />
  </div>
</div>
