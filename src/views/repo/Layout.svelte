<script lang="ts">
  import type { RepoInfo } from "@bindings/RepoInfo";

  import Border from "@app/components/Border.svelte";
  import CopyableId from "@app/components/CopyableId.svelte";
  import Header from "@app/components/Header.svelte";
  import Icon from "@app/components/Icon.svelte";
  import NakedButton from "@app/components/NakedButton.svelte";
  import RepoHeader from "@app/components/RepoHeader.svelte";

  export let repo: RepoInfo;
  export let selfDid: string;

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
    margin-bottom: 1rem;
  }

  .sidebar {
    grid-column: 1 / 2;
    margin-left: 1rem;
    margin-right: 0.5rem;
    min-width: 14rem;
  }

  .content {
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
        <CopyableId id={repo.rid} />
      </svelte:fragment>
      <svelte:fragment slot="breadcrumbs">
        <slot name="breadcrumbs" />
      </svelte:fragment>
    </Header>
  </div>

  <div class="sidebar" class:hidden>
    <Border
      hoverable={false}
      variant="ghost"
      styleWidth="100%"
      styleHeight="32px">
      <RepoHeader {repo} {selfDid} emphasizedTitle={false} />
    </Border>

    <slot name="sidebar" />
  </div>

  <div class="content">
    <slot />
  </div>
</div>
