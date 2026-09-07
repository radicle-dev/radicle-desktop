<script lang="ts" module>
  // Mermaid is heavy and `initialize` mutates a process wide singleton, so load
  // it lazily and share the one instance across every Markdown component.
  let mermaidPromise: Promise<typeof import("mermaid").default> | undefined;
  let mermaidTheme: "dark" | "light" | undefined;

  async function loadMermaid(theme: "dark" | "light") {
    mermaidPromise ??= import("mermaid").then(
      ({ default: mermaid }) => mermaid,
    );
    const mermaid = await mermaidPromise;

    // `initialize` replaces the entire site config, so it is passed in full and
    // only re-run when the app theme changed since the last diagram.
    if (mermaidTheme !== theme) {
      mermaid.initialize({
        startOnLoad: false,
        securityLevel: "strict",
        // Render our own warning instead of letting mermaid append its
        // "Syntax error" graphic to the document.
        suppressErrorRendering: true,
        theme: theme === "dark" ? "dark" : "default",
      });
      mermaidTheme = theme;
    }

    return mermaid;
  }

  const alertIcons = {
    note: "guide",
    tip: "lightbulb",
    important: "comment",
    warning: "warning",
    caution: "stop",
  } as const;
</script>

<script lang="ts">
  import type { Embed } from "@bindings/cob/Embed";
  import type { Blob as SourceBlob } from "@bindings/source/Blob";

  import dompurify from "dompurify";
  import { toDom } from "hast-util-to-dom";
  import { mount, tick, unmount } from "svelte";
  import { get } from "svelte/store";

  import { parseFrontmatter } from "@app/lib/frontmatter";
  import { invoke } from "@app/lib/invoke";
  import {
    markdownWithExtensions,
    Renderer,
    sanitizeConfig,
  } from "@app/lib/markdown";
  import { alertVariants } from "@app/lib/markdown";
  import { highlight } from "@app/lib/syntax";
  import {
    canonicalize,
    canonicalizeGithubImageUrl,
    isCommit,
    isUrl,
    scrollIntoView,
    twemoji,
  } from "@app/lib/utils";

  import Icon from "@app/components/Icon.svelte";
  import { theme } from "@app/components/ThemeSwitch.svelte";

  interface Props {
    rid?: string;
    content: string;
    // Path of the document being rendered, used to resolve relative links to
    // other files in the repository.
    path?: string;
    // Commit the document is being read at, without which repository-relative
    // images cannot be resolved.
    sha?: string;
    // If true, add <br> on a single line break
    breaks?: boolean;
  }

  const {
    rid = "",
    content,
    path = "",
    sha = undefined,
    breaks = false,
  }: Props = $props();

  let container: HTMLElement;

  const doc = $derived(parseFrontmatter(content));
  const frontMatter = $derived.by(() => {
    try {
      return Object.entries(doc.data).filter(
        ([, val]) => typeof val === "string" || typeof val === "number",
      );
    } catch (error) {
      console.error("Not able to parse frontmatter: ", error);
    }
  });

  function render(content: string): string {
    return dompurify.sanitize(
      markdownWithExtensions.parse(content, {
        renderer: new Renderer(),
        breaks,
      }) as string,
      sanitizeConfig,
    );
  }

  $effect(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    content;

    // Components mounted into the rendered markup, torn down when `content`
    // changes and the markup they live in is replaced.
    const mounted: Record<string, unknown>[] = [];

    void tick().then(async () => {
      // Don't run this if the component hasn't mounted yet.
      if (container === null) {
        return;
      }

      for (const e of container.querySelectorAll("a")) {
        try {
          const url = new URL(e.href);
          if (url.origin !== window.origin) {
            e.target = "_blank";
            e.rel = "noopener noreferrer";
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

        // Iterate over all links, and try to add a base64 preview beneath it.
        const href = e.getAttribute("href");

        // If the markdown link is an oid embed
        if (href && isCommit(href)) {
          e.onclick = event => {
            event.preventDefault();
            invoke("save_embed_to_disk", {
              rid,
              oid: href,
              name: e.innerText,
            }).catch(console.error);
          };
          void invoke<Embed>("get_embed", {
            rid,
            name: e.innerText,
            oid: href,
          })
            .then(({ mimeType, content }) => {
              const buffer = Buffer.from(content);
              const blob = new Blob([buffer]);
              const url = URL.createObjectURL(blob);
              // Embed an img element below the link
              if (mimeType?.startsWith("image")) {
                const element = document.createElement("img");
                element.setAttribute("src", url);
                element.style.display = "block";
                e.style.display = "block";
                e.insertAdjacentElement("afterend", element);
                // Embed an iframe to display pdf correctly element below the link
              } else if (mimeType?.startsWith("application")) {
                const element = document.createElement("embed");
                element.setAttribute("src", url);
                element.type = mimeType;
                element.style.overflow = "scroll";
                element.style.height = "40rem";
                element.style.overscrollBehavior = "contain";
                e.style.display = "block";
                e.insertAdjacentElement("afterend", element);
              } else if (mimeType?.startsWith("video")) {
                const element = document.createElement("video");
                const node = document.createElement("source");
                node.src = url;
                element.controls = true;
                node.type = mimeType;
                element.style.width = "100%";
                e.style.display = "block";
                element.appendChild(node);
                e.insertAdjacentElement("afterend", element);
              } else if (mimeType?.startsWith("audio")) {
                const element = document.createElement("audio");
                element.style.display = "block";
                element.src = url;
                element.controls = true;
                e.style.display = "block";
                e.insertAdjacentElement("afterend", element);
              } else {
                console.warn(`Not able to provide a preview for this file.`);
              }
            })
            .catch(console.error);
        }
      }

      // Images referring to a path in the repository rather than an absolute
      // URL are read out of storage and handed to the webview as a blob.
      for (const image of container.querySelectorAll("img")) {
        const src = image.getAttribute("src");
        if (!src || image.classList.contains("txt-emoji")) {
          continue;
        }

        if (isUrl(src)) {
          image.setAttribute("src", canonicalizeGithubImageUrl(src));
          continue;
        }

        if (src.startsWith("data:") || !sha) {
          continue;
        }

        void invoke<SourceBlob>("repo_blob", {
          rid,
          path: canonicalize(src, path),
          sha,
        })
          .then(blob => {
            const bytes = blob.binary
              ? Uint8Array.from(atob(blob.content), c => c.charCodeAt(0))
              : new TextEncoder().encode(blob.content);

            image.setAttribute(
              "src",
              URL.createObjectURL(new Blob([bytes], { type: blob.mimeType })),
            );
          })
          .catch(() => console.warn("Not able to load image", src));
      }

      for (const title of container.querySelectorAll("p.alert-title")) {
        const variant = alertVariants.find(name =>
          title.parentElement?.classList.contains(`alert-${name}`),
        );
        if (!variant) continue;

        mounted.push(
          mount(Icon, {
            target: title,
            anchor: title.firstChild ?? undefined,
            props: { name: alertIcons[variant] },
          }),
        );
      }

      // Replaces code blocks in the background with highlighted code.
      const prefix = "language-";
      const nodes = Array.from(container.querySelectorAll("pre code"));

      const treeChanges: Promise<void>[] = [];

      for (const node of nodes) {
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

        const language = className.slice(prefix.length);

        if (language === "mermaid") {
          const mermaid = await loadMermaid(get(theme));
          try {
            const { svg } = await mermaid.render(
              `mermaid-${crypto.randomUUID()}`,
              node.textContent ?? "",
            );
            // The component may have re-rendered or unmounted while we were
            // awaiting, in which case `preWrapper` is detached and there is
            // nothing left to replace.
            if (preWrapper.isConnected) {
              const diagram = document.createElement("div");
              diagram.classList.add("mermaid-diagram");
              diagram.innerHTML = svg;
              preWrapper.replaceWith(diagram);
            }
          } catch (error) {
            console.warn("Not able to render mermaid diagram", error);
            if (preWrapper.isConnected) {
              const warning = document.createElement("div");
              warning.classList.add("mermaid-error");
              mounted.push(
                mount(Icon, { target: warning, props: { name: "warning" } }),
              );
              const message = document.createElement("span");
              message.textContent = "Couldn't render diagram";
              warning.appendChild(message);
              preWrapper.before(warning);
            }
          }
          continue;
        }

        treeChanges.push(
          highlight(node.textContent ?? "", language)
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

    return () => {
      for (const component of mounted) {
        void unmount(component);
      }
    };
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
    font: var(--txt-body-s-regular);
    border: 1px dashed var(--color-border-mid);
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
    font: var(--txt-heading-l);
    padding: 1rem 0 0.5rem 0;
    margin: 0 0 0.75rem;
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .markdown :global(h2) {
    font: var(--txt-heading-m);
    padding: 0.25rem 0;
    margin: 2rem 0 0.5rem;
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .markdown :global(.pre-wrapper) {
    position: relative;
    margin: 1rem 0;
  }

  .markdown :global(radicle-clipboard) {
    display: none;
    position: absolute;
    right: 0.75rem;
    top: 0.75rem;
  }

  .markdown :global(radicle-clipboard) {
    background-color: var(--color-surface-subtle);
  }

  .markdown :global(.pre-wrapper:hover > radicle-clipboard) {
    display: flex;
  }

  .markdown :global(h3) {
    font: var(--txt-heading-m);
    padding: 0.5rem 0;
    margin: 1rem 0 0.25rem;
  }

  .markdown :global(h4) {
    font: var(--txt-body-l-semibold);
    padding: 0.5rem 0;
    margin: 1rem 0 0.125rem;
  }

  .markdown :global(h5),
  .markdown :global(h6) {
    font: var(--txt-body-m-semibold);
    padding: 0.35rem 0;
    margin: 1rem 0 0.125rem;
  }

  .markdown :global(h6) {
    color: var(--color-text-secondary);
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
    color: var(--color-text-secondary);
  }
  .markdown :global(li.task-item:not(:last-child)) {
    margin-bottom: 0.25rem;
  }

  .markdown :global(blockquote) {
    color: var(--color-text-secondary);
    border-left: 0.3rem solid var(--color-surface-subtle);
    padding: 0 0 0 1rem;
    margin: 1rem 0 1rem 0;
  }

  .markdown :global(strong) {
    font-weight: 600;
  }

  .markdown :global(.footnote-ref) {
    vertical-align: top;
    position: relative;
    top: -0.4rem;
  }
  .markdown :global(.footnote-ref),
  .markdown :global(.footnote > .marker),
  .markdown :global(.footnote > .ref-arrow) {
    color: var(--color-text-secondary);
  }
  .markdown :global(.footnote-ref:hover),
  .markdown :global(.footnote .ref-arrow:hover) {
    color: var(--color-surface-base);
  }
  .markdown :global(.footnote) {
    margin-bottom: 0;
  }

  .markdown :global(img) {
    border-style: none;
    max-width: 100%;
  }

  .markdown :global(code) {
    font: var(--txt-code-regular);
    background-color: var(--color-surface-subtle);
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
    font: var(--txt-code-regular);
    background-color: var(--color-surface-subtle);
    padding: 1rem !important;
    overflow: scroll;
    scrollbar-width: none;
    border-radius: var(--border-radius-sm);
  }

  .markdown :global(pre::-webkit-scrollbar) {
    display: none;
  }

  .markdown :global(a),
  .markdown :global(a > code) {
    color: var(--color-text-primary);
    background: none;
    padding: 0;
  }
  .markdown :global(a) {
    text-decoration: underline;
    text-decoration-color: var(--color-text-secondary);
  }
  .markdown :global(a.no-underline) {
    text-decoration: none;
  }
  .markdown :global(a:hover) {
    text-decoration-color: var(--color-text-primary);
  }

  .markdown :global(hr) {
    height: 0;
    margin: 1rem 0;
    overflow: hidden;
    background: transparent;
    border: 0;
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .markdown :global(ol) {
    line-height: 1.625rem;
    list-style-type: decimal;
    margin-bottom: 1rem;
    padding-left: 2.5rem;
  }

  .markdown :global(ul) {
    line-height: 1.625rem;
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
    box-shadow: 0 0 0 1px var(--color-border-subtle);
    overflow: hidden;
  }
  .markdown :global(td) {
    text-align: left;
    border: 1px solid var(--color-border-subtle);
    padding: 0.5rem 1rem;
    word-break: normal;
    overflow-wrap: normal;
  }
  .markdown :global(tr:nth-child(even)) {
    background-color: var(--color-surface-base);
  }
  .markdown :global(th) {
    text-align: center;
    padding: 0.5rem 1rem;
    word-break: normal;
    overflow-wrap: normal;
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

  .markdown :global(.alert) {
    border-left: 0.3rem solid var(--alert-color);
    padding: 0 0 0 1rem;
    margin: 1rem 0;
  }
  .markdown :global(.alert > .alert-title) {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: var(--alert-color);
    font: var(--txt-body-m-semibold);
    margin-bottom: 0.375rem;
  }
  .markdown :global(.alert > :last-child) {
    margin-bottom: 0;
  }
  .markdown :global(.alert-note) {
    --alert-color: var(--color-feedback-info-text);
  }
  .markdown :global(.alert-tip) {
    --alert-color: var(--color-feedback-success-text);
  }
  .markdown :global(.alert-important) {
    --alert-color: var(--color-feedback-important-text);
  }
  .markdown :global(.alert-warning) {
    --alert-color: var(--color-feedback-warning-text);
  }
  .markdown :global(.alert-caution) {
    --alert-color: var(--color-feedback-error-text);
  }

  .markdown :global(.mermaid-diagram) {
    margin: 1rem 0;
    text-align: center;
  }
  .markdown :global(.mermaid-diagram svg) {
    max-width: 100%;
    height: auto;
  }
  .markdown :global(.mermaid-error) {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    color: var(--color-feedback-warning-text);
    font: var(--txt-body-s-regular);
    margin: 1rem 0 0.25rem;
  }
</style>

{#if frontMatter && frontMatter.length > 0}
  <div class="front-matter">
    <table>
      <tbody>
        {#each frontMatter as [key, val]}
          <tr>
            <td><span class="txt-body-l-semibold">{key}</span></td>
            <td>{val}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/if}

<div class="markdown" bind:this={container} use:twemoji={{ exclude: ["21a9"] }}>
  {@html render(doc.content)}
</div>
