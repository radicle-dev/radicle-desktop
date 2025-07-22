<script lang="ts" module>
  let currentPath = $state("");

  export function getCurrentPath() {
    return currentPath;
  }
</script>

<script lang="ts">
  import type { Config } from "@bindings/config/Config";
  import type { Readme } from "@bindings/repo/Readme";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";
  import type { Blob } from "@bindings/source/Blob";
  import type { Tree } from "@bindings/source/Tree";

  import { toHtml } from "hast-util-to-html";
  import { capitalize } from "lodash";
  import { useOverlayScrollbars } from "overlayscrollbars-svelte";

  import { invoke, InvokeError } from "@app/lib/invoke";
  import { highlight } from "@app/lib/syntax";

  import Border from "@app/components/Border.svelte";
  import CheckoutRepoButton from "@app/components/CheckoutRepoButton.svelte";
  import File from "@app/components/File.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import Markdown from "@app/components/Markdown.svelte";
  import NodeBreadcrumb from "@app/components/NodeBreadcrumb.svelte";
  import Path from "@app/components/Path.svelte";
  import PreviewSwitch from "@app/components/PreviewSwitch.svelte";
  import RepoHomeSecondColumn from "@app/components/RepoHomeSecondColumn.svelte";
  import RepoMetadata from "@app/components/RepoMetadata.svelte";
  import Layout from "@app/views/repo/Layout.svelte";
  import RepoBreadcrumb from "@app/views/repo/RepoBreadcrumb.svelte";

  interface Props {
    config: Config;
    tree: Tree;
    repo: RepoInfo;
    readme: Readme | null;
    notificationCount: number;
  }

  /* eslint-disable prefer-const */
  let { config, tree, readme, repo, notificationCount }: Props = $props();
  /* eslint-enable prefer-const */

  let codeElement: HTMLElement | undefined = $state();
  let preview = $state(true);
  let error: InvokeError | undefined = $state();

  $effect(() => {
    currentPath = readme?.path || "";
  });

  function isMarkdownPath(path: string): boolean {
    return /\.(md|mkd|markdown)$/i.test(path);
  }

  async function fetchTree(path: string) {
    return await invoke<Tree>("repo_tree", { rid: repo.rid, path });
  }

  async function fetchBlob(path: string) {
    try {
      blob = await invoke<Blob>("repo_blob", { rid: repo.rid, path });
      currentPath = path;
      error = undefined;
    } catch (err) {
      if (err instanceof InvokeError) {
        error = err;
      }
      currentPath = path;
    }
    return;
  }

  $effect(() => {
    if (codeElement) {
      const [initialize] = useOverlayScrollbars({
        options: () => ({
          scrollbars: { theme: "global-os-theme-radicle", autoHide: "scroll" },
        }),
        defer: true,
      });

      initialize({ target: codeElement });
    }
  });

  $effect(() => {
    preview = isMarkdownPath(currentPath);
  });

  const project = $derived(repo.payloads["xyz.radicle.project"]!);
  let blob: Blob | Readme | null = $state(readme);
  const showLineNumbers = $derived(
    blob && !blob.binary && blob.content.trim() !== "" && !preview && !error,
  );
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
  .line-column {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
  }
  .blob {
    display: flex;
    gap: 1rem;
    padding: 0.5rem 1rem;
    overflow: hidden;
  }
  .blob-placeholder {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 1rem 0;
  }
  .code,
  .commit-msg {
    -webkit-touch-callout: initial;
    -webkit-user-select: text;
    user-select: text;
    cursor: text;
  }
</style>

<Layout
  {config}
  hideSidebar
  styleSecondColumnOverflow="visible"
  {notificationCount}>
  {#snippet breadcrumbs()}
    <NodeBreadcrumb {config} />
    <Icon name="chevron-right" />
    <RepoBreadcrumb name={project.data.name} rid={repo.rid} />
  {/snippet}

  {#snippet secondColumn()}
    <RepoHomeSecondColumn {repo} {tree} {fetchBlob} {fetchTree} />
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
        {#if blob === null}
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
                <Path fullPath={currentPath} />
              </div>
            {/snippet}

            {#snippet rightHeader()}
              {#if blob}
                <Border
                  styleMaxWidth="fit-content"
                  variant="float"
                  styleBackgroundColor="var(--color-background-float)"
                  stylePadding="0 0.5rem"
                  styleAlignItems="center"
                  styleAlignSelf="flex-end">
                  <Id
                    variant="commit"
                    id={blob.commit.id}
                    clipboard={blob.commit.id} />
                  <span class="commit-msg txt-overflow" style:max-width="20rem">
                    {blob.commit.message}
                  </span>
                </Border>
              {/if}

              {#if isMarkdownPath(currentPath)}
                <PreviewSwitch bind:preview />
              {/if}
            {/snippet}

            <div class="blob">
              <div class="line-column">
                {#if showLineNumbers}
                  {#each blob.content
                    .trimEnd()
                    .split("\n")
                    .map((_, index) => index) as line}
                    <div class="txt-missing txt-monospace txt-small">
                      {line + 1}
                    </div>
                  {/each}
                {/if}
              </div>
              <div style:width="100%" bind:this={codeElement}>
                {#if blob.binary}
                  {#if blob.mimeType.startsWith("image")}
                    <img
                      src={`data:${blob.mimeType};base64,${blob.content}`}
                      alt={`Preview of ${blob.id}`} />
                  {:else}
                    <div class="txt-small blob-placeholder txt-missing">
                      <Icon name="file" size="32" />
                      <span>Binary file</span>
                    </div>
                  {/if}
                {:else if preview}
                  <div style:margin-top="1rem">
                    <Markdown content={blob.content} />
                  </div>
                {:else if blob.content.trim() === ""}
                  <div class="txt-small blob-placeholder txt-missing">
                    <Icon name="none" size="32" />
                    <span>Empty file</span>
                  </div>
                {:else if error}
                  <div class="txt-small blob-placeholder txt-missing">
                    <Icon name="warning" size="32" />
                    {#if error.code === "PayloadError.TooLarge"}
                      <span>File size exceeds limit of 10 MB.</span>
                    {:else}
                      <span>{capitalize(error.message)}</span>
                    {/if}
                  </div>
                {:else}
                  <code>
                    <pre
                      class="code txt-small"
                      style:margin="0"
                      style:padding="0">{#await highlight(blob.content, currentPath
                          .split(".")
                          .at(-1) || "raw")}{blob.content}{:then tree}{@html toHtml(
                          tree,
                        )}{/await}</pre>
                  </code>
                {/if}
              </div>
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
