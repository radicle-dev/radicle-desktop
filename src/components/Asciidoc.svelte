<script lang="ts">
  import { toDom } from "hast-util-to-dom";
  import { tick } from "svelte";

  import type { AsciidocSource } from "@app/lib/asciidoc";
  import { renderAsciidoc, resolveRepoPath } from "@app/lib/asciidoc";
  import { highlight } from "@app/lib/syntax";
  import { scrollIntoView, twemoji } from "@app/lib/utils";

  interface Props extends AsciidocSource {
    content: string;
    onNavigate?: (path: string) => void;
  }

  const { content, rid, sha, path, onNavigate }: Props = $props();

  let container: HTMLElement | undefined = $state();
  let html = $state("");
  let error: string | undefined = $state();

  $effect(() => {
    const text = content;
    const source = { rid, sha, path };
    let stale = false;

    renderAsciidoc(text, source)
      .then(result => {
        if (!stale) {
          html = result;
          error = undefined;
        }
      })
      .catch(err => {
        if (!stale) {
          html = "";
          error = err instanceof Error ? err.message : String(err);
        }
      });

    return () => {
      stale = true;
    };
  });

  $effect(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    html;

    void tick().then(() => {
      if (!container) {
        return;
      }

      for (const anchor of container.querySelectorAll("a")) {
        const href = anchor.getAttribute("href");
        if (!href || href.startsWith("#")) {
          continue;
        }

        // A link relative to the document is a path in the repository, not in
        // the app. Following it would navigate the window away, so open the
        // file in the source view instead.
        const repoPath = resolveRepoPath(href, path);
        if (repoPath) {
          anchor.onclick = event => {
            event.preventDefault();
            onNavigate?.(repoPath);
          };
        } else {
          anchor.target = "_blank";
          anchor.rel = "noopener noreferrer";
        }
      }

      const prefix = "language-";
      const treeChanges: Promise<void>[] = [];

      for (const node of container.querySelectorAll("pre code")) {
        const preElement = node.parentElement as HTMLElement;
        const copyButton = document.createElement("radicle-clipboard");
        copyButton.setAttribute("text", node.textContent || "");
        const preWrapper = document.createElement("div");
        preWrapper.classList.add("pre-wrapper");
        preElement.parentNode?.insertBefore(preWrapper, preElement);
        preWrapper.appendChild(preElement);
        preWrapper.appendChild(copyButton);

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

      void Promise.allSettled(treeChanges);

      if (window.location.hash) {
        scrollIntoView(window.location.hash.substring(1));
      }
    });
  });
</script>

<style>
  .asciidoc {
    word-break: break-word;
    -webkit-touch-callout: initial;
    -webkit-user-select: text;
    user-select: text;
  }

  .asciidoc :global(h1) {
    font: var(--txt-heading-l);
    padding: 1rem 0 0.5rem 0;
    margin: 0 0 0.75rem;
    border-bottom: 1px solid var(--color-border-subtle);
  }
  .asciidoc :global(h2) {
    font: var(--txt-heading-m);
    padding: 0.25rem 0;
    margin: 2rem 0 0.5rem;
    border-bottom: 1px solid var(--color-border-subtle);
  }
  .asciidoc :global(h3) {
    font: var(--txt-heading-m);
    padding: 0.5rem 0;
    margin: 1rem 0 0.25rem;
  }
  .asciidoc :global(h4) {
    font: var(--txt-body-l-semibold);
    padding: 0.5rem 0;
    margin: 1rem 0 0.125rem;
  }
  .asciidoc :global(h5),
  .asciidoc :global(h6) {
    font: var(--txt-body-m-semibold);
    padding: 0.35rem 0;
    margin: 1rem 0 0.125rem;
  }
  .asciidoc :global(h6) {
    color: var(--color-text-secondary);
  }

  .asciidoc :global(p) {
    line-height: 1.625rem;
    margin: 0 0 0.625rem;
  }
  .asciidoc :global(strong) {
    font-weight: 600;
  }
  .asciidoc :global(hr) {
    height: 0;
    margin: 1rem 0;
    background: transparent;
    border: 0;
    border-bottom: 1px solid var(--color-border-subtle);
  }
  .asciidoc :global(a) {
    color: var(--color-text-primary);
    text-decoration: underline;
    text-decoration-color: var(--color-text-secondary);
  }
  .asciidoc :global(a:hover) {
    text-decoration-color: var(--color-text-primary);
  }
  .asciidoc :global(.image img),
  .asciidoc :global(.imageblock img) {
    border-style: none;
    max-width: 100%;
  }

  .asciidoc :global(code) {
    font: var(--txt-code-regular);
    background-color: var(--color-surface-subtle);
    padding: 0.125rem 0.25rem;
  }
  .asciidoc :global(:not(pre) > code) {
    font-size: inherit;
  }
  .asciidoc :global(pre) {
    font: var(--txt-code-regular);
    background-color: var(--color-surface-subtle);
    padding: 1rem;
    margin: 0;
    overflow: scroll;
    scrollbar-width: none;
    border-radius: var(--border-radius-sm);
  }
  .asciidoc :global(pre::-webkit-scrollbar) {
    display: none;
  }
  .asciidoc :global(pre > code) {
    background: none;
    padding: 0;
  }
  .asciidoc :global(.pre-wrapper) {
    position: relative;
    margin: 1rem 0;
  }
  .asciidoc :global(radicle-clipboard) {
    display: none;
    position: absolute;
    right: 0.75rem;
    top: 0.75rem;
    background-color: var(--color-surface-subtle);
  }
  .asciidoc :global(.pre-wrapper:hover > radicle-clipboard) {
    display: flex;
  }

  /* Asciidoctor wraps every block in a div and every list item's text in a
     paragraph, so the inner margins have to be reset. */
  .asciidoc :global(li > p),
  .asciidoc :global(dd > p),
  .asciidoc :global(td > p) {
    margin-bottom: 0;
  }
  .asciidoc :global(ul),
  .asciidoc :global(ol) {
    line-height: 1.625rem;
    margin: 0 0 1rem;
  }
  .asciidoc :global(ul) {
    list-style-type: inherit;
    padding-left: 1.25rem;
  }
  .asciidoc :global(ol) {
    list-style-type: decimal;
    padding-left: 2.5rem;
  }
  .asciidoc :global(li > .ulist),
  .asciidoc :global(li > .olist) {
    margin-bottom: 0;
  }
  .asciidoc :global(li > .ulist ul),
  .asciidoc :global(li > .olist ol) {
    margin-bottom: 0;
  }
  .asciidoc :global(dl dt) {
    font-style: italic;
    margin-top: 1rem;
  }
  .asciidoc :global(dl dd) {
    margin: 0 0 0 2rem;
  }

  .asciidoc :global(table.tableblock) {
    margin: 1.5rem 0;
    border-collapse: collapse;
    border-style: hidden;
    box-shadow: 0 0 0 1px var(--color-border-subtle);
  }
  /* Asciidoctor sizes columns with a colgroup. Without a fixed layout an
     unbreakable token in a cell widens its column past the viewport. */
  .asciidoc :global(table.tableblock.stretch) {
    width: 100%;
    table-layout: fixed;
  }
  .asciidoc :global(td.tableblock) {
    text-align: left;
    border: 1px solid var(--color-border-subtle);
    padding: 0.5rem 1rem;
    overflow-wrap: break-word;
  }
  .asciidoc :global(table.tableblock tr:nth-child(even)) {
    background-color: var(--color-surface-base);
  }
  .asciidoc :global(th.tableblock) {
    text-align: left;
    padding: 0.5rem 1rem;
    overflow-wrap: break-word;
  }
  /* An AsciiDoc cell holds a nested document, so its blocks need the same
     margin reset as the top level and must stay inside the column. */
  .asciidoc :global(td.tableblock .pre-wrapper) {
    margin: 0;
  }
  .asciidoc :global(td.tableblock pre) {
    max-width: 100%;
  }
  .asciidoc :global(td.tableblock #footnotes) {
    margin-top: 0.5rem;
  }
  .asciidoc :global(td.tableblock #footnotes hr) {
    display: none;
  }
  .asciidoc :global(td.tableblock > .content > :last-child) {
    margin-bottom: 0;
  }
  .asciidoc :global(caption.title) {
    font: var(--txt-body-s-regular);
    color: var(--color-text-secondary);
    text-align: left;
    padding-bottom: 0.25rem;
  }

  .asciidoc :global(.admonitionblock) {
    margin: 1rem 0;
  }
  .asciidoc :global(.admonitionblock > table) {
    border-collapse: collapse;
    width: 100%;
  }
  .asciidoc :global(.admonitionblock td.icon) {
    width: 6rem;
    padding: 0.5rem 1rem;
    vertical-align: top;
    border-left: 0.3rem solid var(--color-border-mid);
    background-color: var(--color-surface-subtle);
  }
  .asciidoc :global(.admonitionblock.warning td.icon),
  .asciidoc :global(.admonitionblock.caution td.icon) {
    border-left-color: var(--color-feedback-warning-border);
  }
  .asciidoc :global(.admonitionblock.important td.icon) {
    border-left-color: var(--color-feedback-error-border);
  }
  .asciidoc :global(.admonitionblock td.icon > .title) {
    font: var(--txt-body-m-semibold);
    text-transform: uppercase;
  }
  .asciidoc :global(.admonitionblock td.content) {
    padding: 0.5rem 1rem;
    background-color: var(--color-surface-subtle);
  }

  .asciidoc :global(blockquote) {
    color: var(--color-text-secondary);
    border-left: 0.3rem solid var(--color-surface-subtle);
    padding: 0 0 0 1rem;
    margin: 1rem 0;
  }
  .asciidoc :global(.quoteblock .attribution) {
    font: var(--txt-body-s-regular);
    color: var(--color-text-secondary);
    margin: 0 0 1rem 1.3rem;
  }
  .asciidoc :global(.sidebarblock) {
    background-color: var(--color-surface-subtle);
    border-radius: var(--border-radius-sm);
    padding: 1rem;
    margin: 1rem 0;
  }

  .asciidoc :global(.listingblock > .title),
  .asciidoc :global(.literalblock > .title),
  .asciidoc :global(.imageblock > .title),
  .asciidoc :global(.exampleblock > .title) {
    font: var(--txt-body-s-regular);
    color: var(--color-text-secondary);
    margin-bottom: 0.25rem;
  }

  .asciidoc :global(#toc) {
    border: 1px dashed var(--color-border-mid);
    padding: 0.5rem 1rem;
    margin-bottom: 2rem;
  }
  .asciidoc :global(#toc > .title) {
    font: var(--txt-body-l-semibold);
  }
  .asciidoc :global(#footnotes) {
    font: var(--txt-body-s-regular);
    color: var(--color-text-secondary);
    margin-top: 2rem;
  }
  .asciidoc :global(sup.footnote) {
    vertical-align: top;
    position: relative;
    top: -0.4rem;
  }

  .asciidoc :global(> :last-child) {
    margin-bottom: 0;
  }
  /* Asciidoctor nests every block in a wrapper div, so this has to stay
     scoped to the top of the document instead of every first child. */
  .asciidoc :global(> :first-child) {
    margin-top: 0;
    padding-top: 0;
  }
</style>

{#if error}
  <div class="txt-missing txt-body-m-regular">
    Not able to render AsciiDoc: {error}
  </div>
{:else}
  <div
    class="asciidoc"
    bind:this={container}
    use:twemoji={{ exclude: ["21a9"] }}>
    {@html html}
  </div>
{/if}
