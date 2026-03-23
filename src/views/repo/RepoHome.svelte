<script lang="ts">
  import type { Readme } from "@bindings/repo/Readme";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";
  import type { Blob } from "@bindings/source/Blob";
  import type { Tree } from "@bindings/source/Tree";

  import { toHtml } from "hast-util-to-html";
  import { capitalize } from "lodash";
  import { useOverlayScrollbars } from "overlayscrollbars-svelte";

  import { invoke, InvokeError } from "@app/lib/invoke";
  import { highlight } from "@app/lib/syntax";

  import FileBlock from "@app/components/FileBlock.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import Markdown from "@app/components/Markdown.svelte";
  import Path from "@app/components/Path.svelte";
  import PreviewSwitch from "@app/components/PreviewSwitch.svelte";
  import RepoHeader from "@app/components/RepoHeader.svelte";
  import ScrollArea from "@app/components/ScrollArea.svelte";
  import TreeComponent from "@app/components/Tree.svelte";
  import Layout from "@app/views/repo/Layout.svelte";

  interface Props {
    tree: Tree;
    repo: RepoInfo;
    readme: Readme | null;
  }

  /* eslint-disable prefer-const */
  let { tree, readme, repo }: Props = $props();
  /* eslint-enable prefer-const */

  let currentPath = $state("");
  let codeElement: HTMLElement | undefined = $state();
  let preview = $state(true);
  let error: InvokeError | undefined = $state();

  $effect(() => {
    blob = readme;
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

  let blob: Blob | Readme | null = $state(readme);
  const showLineNumbers = $derived(
    blob && !blob.binary && blob.content.trim() !== "" && !preview && !error,
  );
</script>

<style>
  .content {
    height: 100%;
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
    justify-content: center;
    min-height: calc(100dvh - 7rem);
  }
  .code,
  .commit-msg {
    -webkit-touch-callout: initial;
    -webkit-user-select: text;
    user-select: text;
    cursor: text;
  }
</style>

<Layout selfScroll>
  <div
    class="content"
    style:display="flex"
    style:flex-direction="column"
    style:height="100%">
    <RepoHeader {repo} />
    <div
      style:display="grid"
      style:grid-template-columns="16.5rem 1fr"
      style:grid-template-rows="100%"
      style:flex="1"
      style:min-height="0">
      <div
        style:display="flex"
        style:flex-direction="column"
        style:height="100%"
        style:min-height="0"
        style:overflow="hidden">
        {#if tree.entries.length > 0}
          <ScrollArea
            style="border-right: 1px solid var(--color-border-subtle); flex: 1; min-height: 0; width: 100%; padding: 0.5rem;">
            <TreeComponent {tree} {currentPath} {fetchTree} {fetchBlob} />
          </ScrollArea>
        {/if}
      </div>
      <ScrollArea style="height: 100%; min-width: 0;">
        <div class="container">
          <div style:min-width="0">
            {#if blob === null}
              <div
                style:display="flex"
                style:min-height="calc(100dvh - 7rem)"
                style:gap="0.5rem"
                style:align-items="center"
                style:background-color="var(--color-surface-canvas)"
                style:padding="1rem">
                <div
                  class="global-flex txt-missing txt-body-m-regular"
                  style:width="100%"
                  style:justify-content="center">
                  No README.md
                </div>
              </div>
            {:else}
              <FileBlock expandable={false} sticky={false} border={false}>
                {#snippet leftHeader()}
                  <div style:margin-left="0.5rem">
                    <Path fullPath={currentPath} />
                  </div>
                {/snippet}

                {#snippet rightHeader()}
                  {#if blob}
                    <div
                      style:display="flex"
                      style:gap="0.5rem"
                      style:align-items="center"
                      style:justify-content="center"
                      style:max-width="fit-content">
                      <Id
                        id={blob.commit.id}
                        clipboard={blob.commit.id}
                        placement="bottom-start" />
                      <span
                        class="commit-msg txt-overflow"
                        style:max-width="20rem">
                        {blob.commit.message}
                      </span>
                    </div>
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
                        <div class="txt-missing txt-code-regular">
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
                        <div
                          class="txt-body-m-regular blob-placeholder txt-missing">
                          <span>Binary file</span>
                        </div>
                      {/if}
                    {:else if preview}
                      <div style:margin-top="1rem">
                        <Markdown content={blob.content} />
                      </div>
                    {:else if blob.content.trim() === ""}
                      <div
                        class="txt-body-m-regular blob-placeholder txt-missing">
                        <span>Empty file</span>
                      </div>
                    {:else if error}
                      <div
                        class="txt-body-m-regular blob-placeholder txt-missing">
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
                          class="code txt-code-regular"
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
              </FileBlock>
            {/if}
          </div>
        </div>
      </ScrollArea>
    </div>
  </div>
</Layout>
