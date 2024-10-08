<script lang="ts">
  import dompurify from "dompurify";
  import matter from "@radicle/gray-matter";
  import { afterUpdate } from "svelte";
  import { toDom } from "hast-util-to-dom";

  import { Renderer, markdownWithExtensions } from "@app/lib/markdown";
  import { highlight } from "@app/lib/syntax";
  import { twemoji, scrollIntoView, isCommit } from "@app/lib/utils";
  import { invoke } from "@tauri-apps/api/core";

  export let rid: string;
  export let content: string;
  // If true, add <br> on a single line break
  export let breaks: boolean = false;

  let container: HTMLElement;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let frontMatter: [string, any][] | undefined = undefined;

  $: {
    try {
      const doc = matter(content);
      content = doc.content;
      frontMatter = Object.entries(doc.data).filter(
        ([, val]) => typeof val === "string" || typeof val === "number",
      );
    } catch (error) {
      console.error("Not able to parse frontmatter: ", error);
    }
  }

  function render(content: string): string {
    return dompurify.sanitize(
      markdownWithExtensions.parse(content, {
        renderer: new Renderer(),
        breaks,
      }) as string,
    );
  }

  afterUpdate(async () => {
    for (const e of container.querySelectorAll("a")) {
      try {
        const url = new URL(e.href);
        if (url.origin !== window.origin) {
          e.target = "_blank";
        }
      } catch (e) {
        console.warn("Not able to parse url", e);
      }
      // Don't underline <a> tags that contain images.
      // Make an exception for emojis.
      if (
        e.firstElementChild instanceof HTMLImageElement &&
        !e.firstElementChild.classList.contains("txt-emoji")
      ) {
        e.classList.add("no-underline");
      }
    }

    // Replace standard HTML checkboxes with our custom radicle-icon-small element
    for (const i of container.querySelectorAll('input[type="checkbox"]')) {
      i.parentElement?.classList.add("task-item");

      const checkbox = document.createElement("radicle-icon-small");
      const checked = i.getAttribute("checked");
      checkbox.setAttribute(
        "name",
        checked === null ? "checkbox-unchecked" : "checkbox-checked",
      );
      i.insertAdjacentElement("beforebegin", checkbox);
      i.remove();
    }

    // Iterate over all images, and replace the source with a canonicalized URL
    // pointing at the repos /raw endpoint.
    for (const i of container.querySelectorAll("img")) {
      const imagePath = i.getAttribute("src");

      // If the image is an oid embed
      if (imagePath && isCommit(imagePath)) {
        const base64Content = await invoke<string>("get_file_by_oid", {
          rid,
          oid: imagePath,
        });

        i.setAttribute("src", `data:image/jpeg;base64,${base64Content}`);
        continue;
      }
    }

    // Replaces code blocks in the background with highlighted code.
    const prefix = "language-";
    const nodes = Array.from(document.body.querySelectorAll("pre code"));

    const treeChanges: Promise<void>[] = [];

    for (const node of nodes) {
      const className = Array.from(node.classList).find(name =>
        name.startsWith(prefix),
      );
      if (!className) continue;

      treeChanges.push(
        highlight(node.textContent ?? "", className.slice(prefix.length))
          .then(tree => {
            if (tree) {
              node.replaceChildren(toDom(tree, { fragment: true }));
            }
          })
          .catch(e => console.warn("Not able to highlight code block", e)),
      );
    }

    await Promise.allSettled(treeChanges);

    if (window.location.hash) {
      scrollIntoView(window.location.hash.substring(1));
    }
  });
</script>

<style>
  :global(html) {
    scroll-padding-top: 4rem;
  }
  .markdown {
    word-break: break-word;
    -webkit-touch-callout: initial;
    -webkit-user-select: text;
    user-select: text;
  }
  .front-matter {
    font-size: var(--font-size-tiny);
    font-family: var(--font-family-monospace);
    border: 1px dashed var(--color-border-default);
    padding: 0.5rem;
    margin-bottom: 2rem;
  }
  .front-matter table {
    border-collapse: collapse;
  }
  .front-matter table td {
    padding: 0.125rem 1rem;
  }
  .front-matter table td:first-child {
    padding-left: 0.5rem;
  }

  .markdown :global(h1) {
    font-size: calc(var(--font-size-x-large) * 0.75);
    font-weight: var(--font-weight-semibold);
    padding: 1rem 0 0.5rem 0;
    margin: 0 0 0.75rem;
    border-bottom: 1px solid var(--color-border-hint);
  }

  .markdown :global(h2) {
    font-size: var(--font-size-medium);
    font-weight: var(--font-weight-regular);
    padding: 0.25rem 0;
    margin: 2rem 0 0.5rem;
    border-bottom: 1px solid var(--color-border-hint);
  }

  .markdown :global(.pre-wrapper) {
    position: relative;
    margin: 1rem 0;
  }

  .markdown :global(radicle-clipboard) {
    display: none;
    position: absolute;
    right: 0.5rem;
    top: 0.5rem;
  }

  .markdown :global(radicle-clipboard) {
    background-color: var(--color-fill-ghost);
  }

  .markdown :global(.pre-wrapper:hover > radicle-clipboard) {
    display: flex;
  }

  .markdown :global(h3) {
    font-size: calc(var(--font-size-medium) * 0.9);
    font-weight: var(--font-weight-semibold);
    padding: 0.5rem 0;
    margin: 1rem 0 0.25rem;
  }

  .markdown :global(h4) {
    font-weight: var(--font-weight-semibold);
    font-size: var(--font-size-regular);
    padding: 0.5rem 0;
    margin: 1rem 0 0.125rem;
  }

  .markdown :global(h5),
  .markdown :global(h6) {
    font-weight: var(--font-weight-semibold);
    font-size: var(--font-size-small);
    padding: 0.35rem 0;
    margin: 1rem 0 0.125rem;
  }

  .markdown :global(h6) {
    color: var(--color-foreground-dim);
  }

  .markdown :global(p) {
    line-height: 1.625rem;
    margin-top: 0;
    margin-bottom: 0.625rem;
  }

  .markdown :global(p:only-child) {
    margin-bottom: 0;
  }

  .markdown :global(li.task-item) {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-left: -1.2rem;
    color: var(--color-foreground-dim);
  }
  .markdown :global(li.task-item:not(:last-child)) {
    margin-bottom: 0.25rem;
  }

  .markdown :global(blockquote) {
    color: var(--color-foreground-dim);
    border-left: 0.3rem solid var(--color-fill-ghost);
    padding: 0 0 0 1rem;
    margin: 1rem 0 1rem 0;
  }

  .markdown :global(strong) {
    font-weight: var(--font-weight-semibold);
  }

  .markdown :global(.footnote-ref) {
    vertical-align: top;
    position: relative;
    top: -0.4rem;
  }
  .markdown :global(.footnote-ref),
  .markdown :global(.footnote > .marker),
  .markdown :global(.footnote > .ref-arrow) {
    color: var(--color-foreground-dim);
  }
  .markdown :global(.footnote-ref:hover),
  .markdown :global(.footnote .ref-arrow:hover) {
    color: var(--color-background-default);
  }
  .markdown :global(.footnote) {
    margin-bottom: 0;
  }

  .markdown :global(img) {
    border-style: none;
    max-width: 100%;
  }

  .markdown :global(code) {
    font-family: var(--font-family-monospace);
    font-size: var(--font-size-small);
    background-color: var(--color-fill-ghost);
    padding: 0.125rem 0.25rem;
  }

  .markdown :global(pre > code) {
    background: none;
    padding: 0;
  }

  .markdown :global(:not(pre) > code) {
    font-size: inherit;
  }

  .markdown :global(pre) {
    font-family: var(--font-family-monospace);
    font-size: var(--font-size-regular);
    background-color: var(--color-fill-ghost);
    padding: 1rem !important;
    overflow: scroll;
    scrollbar-width: none;
  }

  .markdown :global(pre::-webkit-scrollbar) {
    display: none;
  }

  .markdown :global(a),
  .markdown :global(a > code) {
    color: var(--color-foreground-contrast);
    background: none;
    padding: 0;
  }
  .markdown :global(a) {
    text-decoration: underline;
    text-decoration-color: var(--color-foreground-dim);
  }
  .markdown :global(a.no-underline) {
    text-decoration: none;
  }
  .markdown :global(a:hover) {
    text-decoration-color: var(--color-foreground-contrast);
  }

  .markdown :global(hr) {
    height: 0;
    margin: 1rem 0;
    overflow: hidden;
    background: transparent;
    border: 0;
    border-bottom: 1px solid var(--color-border-hint);
  }

  .markdown :global(ol) {
    line-height: 1.625;
    list-style-type: decimal;
    margin-bottom: 1rem;
    padding-left: 1.5rem;
  }

  .markdown :global(ul) {
    line-height: 1.625;
    list-style-type: inherit;
    padding-left: 1.25rem;
    margin-bottom: 1rem;
  }
  .markdown :global(.list-content) {
    margin: 1rem 0;
  }
  /* Allows the parent to specify its own bottom margin */
  .markdown :global(> :last-child) {
    margin-bottom: 0;
  }
  .markdown :global(li > ul) {
    margin-bottom: 0rem;
  }
  .markdown :global(li > ol) {
    margin-bottom: 0rem;
  }
  .markdown :global(table) {
    margin: 1.5rem 0;
    border-collapse: collapse;
    border-style: hidden;
    box-shadow: 0 0 0 1px var(--color-border-hint);
    overflow: hidden;
  }
  .markdown :global(td) {
    text-align: left;
    text-overflow: ellipsis;
    border: 1px solid var(--color-border-hint);
    padding: 0.5rem 1rem;
  }
  .markdown :global(tr:nth-child(even)) {
    background-color: var(--color-background-default);
  }
  .markdown :global(th) {
    text-align: center;
    padding: 0.5rem 1rem;
  }

  .markdown :global(*:first-child:not(pre)) {
    padding-top: 0 !important;
  }
  .markdown :global(*:first-child) {
    margin-top: 0 !important;
  }
  .markdown :global(dl dt) {
    font-style: italic;
    margin-top: 1rem;
  }
  .markdown :global(dl dd) {
    margin: 0 0 0 2rem;
  }
</style>

{#if frontMatter && frontMatter.length > 0}
  <div class="front-matter">
    <table>
      {#each frontMatter as [key, val]}
        <!-- svelte-ignore node_invalid_placement_ssr -->
        <tr>
          <td><span class="txt-bold">{key}</span></td>
          <td>{val}</td>
        </tr>
      {/each}
    </table>
  </div>
{/if}

<div class="markdown" bind:this={container} use:twemoji={{ exclude: ["21a9"] }}>
  {@html render(content)}
</div>
