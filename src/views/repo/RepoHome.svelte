<script lang="ts">
  import type { Config } from "@bindings/config/Config";
  import type { Readme } from "@bindings/repo/Readme";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import Border from "@app/components/Border.svelte";
  import CheckoutRepoButton from "@app/components/CheckoutRepoButton.svelte";
  import CopyableId from "@app/components/CopyableId.svelte";
  import File from "@app/components/File.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Layout from "./Layout.svelte";
  import Markdown from "@app/components/Markdown.svelte";
  import Path from "@app/components/Path.svelte";
  import RepoHomeSecondColumn from "@app/components/RepoHomeSecondColumn.svelte";
  import RepoMetadata from "@app/components/RepoMetadata.svelte";

  interface Props {
    config: Config;
    readme: Readme | null;
    repo: RepoInfo;
    notificationCount: number;
  }

  const { config, readme, repo, notificationCount }: Props = $props();

  const project = $derived(repo.payloads["xyz.radicle.project"]!);
</script>

<style>
  .content {
    padding: 1rem 1rem 1rem 0;
  }
  .container {
    display: grid;
    grid-template-columns: 1fr min-content;
    grid-template-areas: "main-content right-sidebar";
    margin-top: 2rem;
  }
</style>

<Layout
  {notificationCount}
  {config}
  hideSidebar
  styleSecondColumnOverflow="visible">
  {#snippet headerCenter()}
    <CopyableId id={repo.rid} />
  {/snippet}

  {#snippet secondColumn()}
    <RepoHomeSecondColumn {repo} />
  {/snippet}

  <div class="content">
    <div class="global-flex">
      <div
        class="txt-medium txt-selectable"
        style:font-weight="var(--font-weight-medium)">
        {project.data.name}
      </div>
      <div class="global-flex" style:margin-left="auto">
        <CheckoutRepoButton rid={repo.rid} />
      </div>
    </div>

    {#if project.data.description !== ""}
      <Markdown rid={repo.rid} breaks content={project.data.description} />
    {/if}

    <div class="global-hide-on-desktop-up" style:margin-top="1rem">
      <RepoMetadata {repo} horizontal />
    </div>

    <div class="container">
      <div style:grid-area="main-content" style:min-width="0">
        {#if readme === null}
          <Border
            variant="ghost"
            stylePadding="1rem"
            styleAlignItems="center"
            styleMinHeight="10rem">
            <div
              class="global-flex txt-missing"
              style:width="100%"
              style:justify-content="center">
              <Icon name="none" />README not found
            </div>
          </Border>
        {:else}
          <File expandable={false} sticky={false}>
            {#snippet leftHeader()}
              <div style:margin-left="0.5rem">
                <Path fullPath={readme.path} />
              </div>
            {/snippet}

            <div style:padding="1rem">
              {#if readme.binary}
                <div
                  class="global-flex txt-missing"
                  style:width="100%"
                  style:justify-content="center">
                  <Icon name="binary" />Binary file
                </div>
              {:else if readme.content.trim() === ""}
                <div
                  class="global-flex txt-missing"
                  style:width="100%"
                  style:justify-content="center">
                  <Icon name="none" />Empty file
                </div>
              {:else}
                <Markdown rid={repo.rid} content={readme.content} />
              {/if}
            </div>
          </File>
        {/if}
      </div>

      <div
        class="global-hide-on-medium-desktop-down"
        style:grid-area="right-sidebar"
        style:margin-left="1rem"
        style:min-width="20rem">
        <RepoMetadata {repo} />
      </div>
    </div>
  </div>
</Layout>
