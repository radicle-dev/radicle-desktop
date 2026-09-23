<script lang="ts">
  import type { Embed } from "@bindings/cob/Embed";

  import { toDom } from "hast-util-to-dom";
  import { mount, tick, unmount } from "svelte";

  import { parseFrontmatter } from "@app/lib/frontmatter";
  import { invoke } from "@app/lib/invoke";
  import {
    isTaskCheckbox,
    renderMarkdown,
    toggleTask,
  } from "@app/lib/markdown";
  import { highlight } from "@app/lib/syntax";
  import { isCommit, scrollIntoView, twemoji } from "@app/lib/utils";

  import Icon from "@app/components/Icon.svelte";

  interface Props {
    rid?: string;
    content: string;
    // If true, add <br> on a single line break
    breaks?: boolean;
    // When set, task-list checkboxes become clickable. Flipping one rewrites
    // that box in the markdown source and hands the whole document back to be
    // saved, so the caller only has to persist what it is given.
    toggleTaskItem?: (content: string) => Promise<void> | void;
  }

  const {
    rid = "",
    content,
    breaks = false,
    toggleTaskItem = undefined,
  }: Props = $props();

  // Guards against a second click landing while the first is still being
  // saved, which would compute the new document from a stale source.
  let taskInFlight = $state(false);
  // The box that had focus when it was clicked. Saving re-renders every box,
  // so focus is handed to its replacement rather than dropped on the page.
  let refocusTask: number | undefined = undefined;
  // Only the first render jumps to the linked anchor. Later ones come from
  // edits such as ticking a box, and should leave the page where it is.
  let scrolledToHash = false;

  let container: HTMLElement;

  // Every change to the content rebuilds the rendered tree, links included.
  // Keeping the preview built for each embed lets a re-render put it straight
  // back instead of fetching and decoding it again, which would leave a gap
  // where the preview was until the new one arrives.
  // eslint-disable-next-line svelte/prefer-svelte-reactivity -- imperative oid→element lookup, never rendered reactively
  const embedPreviews = new Map<
    string,
    { element: HTMLElement; url: string }
  >();

  let destroyed = false;

  $effect(() => () => {
    destroyed = true;
    for (const { url } of embedPreviews.values()) {
      URL.revokeObjectURL(url);
    }
    embedPreviews.clear();
  });

  function createEmbedPreview(
    mimeType: string | null | undefined,
    url: string,
  ): HTMLElement | undefined {
    if (mimeType?.startsWith("image")) {
      const element = document.createElement("img");
      element.setAttribute("src", url);
      element.style.display = "block";
      return element;
    } else if (mimeType?.startsWith("application")) {
      // An embed element is what displays a PDF correctly.
      const element = document.createElement("embed");
      element.setAttribute("src", url);
      element.type = mimeType;
      element.style.overflow = "scroll";
      element.style.height = "40rem";
      element.style.overscrollBehavior = "contain";
      return element;
    } else if (mimeType?.startsWith("video")) {
      const element = document.createElement("video");
      const node = document.createElement("source");
      node.src = url;
      element.controls = true;
      node.type = mimeType;
      element.style.width = "100%";
      element.appendChild(node);
      return element;
    } else if (mimeType?.startsWith("audio")) {
      const element = document.createElement("audio");
      element.style.display = "block";
      element.src = url;
      element.controls = true;
      return element;
    }
  }

  function placeEmbedPreview(link: HTMLAnchorElement, element: HTMLElement) {
    link.style.display = "block";
    // The same embed can be linked more than once in a document, and one
    // element can only sit in one place.
    link.insertAdjacentElement(
      "afterend",
      container.contains(element)
        ? (element.cloneNode(true) as HTMLElement)
        : element,
    );
  }

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

  $effect(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    content;

    const icons: ReturnType<typeof mount>[] = [];

    void tick().then(() => {
      // Don't run this if the component hasn't mounted yet.
      if (container === null) {
        return;
      }

      // Replace native task-list checkboxes with styled boxes. The checkmark
      // is always mounted and hidden by CSS while unchecked, so flipping a box
      // is a single class change.
      // Only boxes rendered from task-list syntax can be written back; one
      // written as raw HTML is styled the same but stays read-only.
      let taskIndex = 0;
      for (const i of container.querySelectorAll('input[type="checkbox"]')) {
        const isTask = isTaskCheckbox(i);
        const index = isTask ? taskIndex++ : -1;
        // In a loose list `marked` wraps the item's content in a `<p>`, so the
        // checkbox's parent is not always the list item itself.
        const item = i.closest("li");
        item?.classList.add("task-item");
        const checked = i.hasAttribute("checked");
        const interactive = toggleTaskItem !== undefined && isTask;
        const box = document.createElement(interactive ? "button" : "span");
        box.classList.add("task-box");
        box.classList.toggle("checked", checked);
        icons.push(mount(Icon, { target: box, props: { name: "checkmark" } }));

        if (interactive && box instanceof HTMLButtonElement) {
          box.type = "button";
          box.setAttribute("role", "checkbox");
          box.setAttribute("aria-checked", String(checked));
          // The item's own text, without that of any nested items.
          const label = item?.cloneNode(true) as HTMLElement | undefined;
          label?.querySelectorAll("ul, ol").forEach(list => list.remove());
          box.setAttribute("aria-label", label?.textContent?.trim() || "Task");
          const setChecked = (value: boolean) => {
            box.classList.toggle("checked", value);
            box.setAttribute("aria-checked", String(value));
          };
          box.onclick = async () => {
            if (taskInFlight || !toggleTaskItem) {
              return;
            }
            const source = content;
            const next = toggleTask(source, index);
            if (next === undefined) {
              console.warn("Not able to find task item in markdown source");
              return;
            }
            taskInFlight = true;
            if (document.activeElement === box) {
              refocusTask = index;
            }
            // Flip the box right away; a successful save reloads the object,
            // which re-renders this from the stored source.
            setChecked(!checked);
            try {
              await toggleTaskItem(next);
            } catch (error) {
              console.error("Not able to save task item: ", error);
            } finally {
              taskInFlight = false;
              // Callers may swallow a failed save rather than throw. Either
              // way the source is unchanged, so nothing re-renders the box and
              // it has to be put back here.
              if (content === source) {
                setChecked(checked);
                refocusTask = undefined;
              }
            }
          };
        }

        i.replaceWith(box);
        if (interactive && index === refocusTask) {
          refocusTask = undefined;
          box.focus();
        }

        // `marked` puts a space between the checkbox and the item's text. The
        // box is spaced by its own margin, so that space only pushes the first
        // line out of line with the wrapped ones.
        const text = box.nextSibling;
        if (text?.nodeType === Node.TEXT_NODE && text.textContent) {
          text.textContent = text.textContent.replace(/^[ \t]+/, "");
        }
      }

      // A list of nothing but task items shows no markers, so it does not need
      // the indent that makes room for them. Mixed lists keep it.
      for (const list of container.querySelectorAll("ul, ol")) {
        const items = Array.from(list.children);
        if (
          items.length > 0 &&
          items.every(c => c.classList.contains("task-item"))
        ) {
          list.classList.add("task-list");
          // The line introducing the list reads as its heading, so it should
          // not also carry a paragraph's worth of space beneath it.
          const lead = list.previousElementSibling;
          if (lead instanceof HTMLParagraphElement) {
            lead.classList.add("task-list-lead");
          }
        }
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
          const cached = embedPreviews.get(href);
          if (cached) {
            placeEmbedPreview(e, cached.element);
          } else {
            void invoke<Embed>("get_embed", {
              rid,
              name: e.innerText,
              oid: href,
            })
              .then(({ mimeType, content }) => {
                if (destroyed) {
                  return;
                }
                // Another render may have fetched this embed in the meantime.
                let preview = embedPreviews.get(href);
                if (!preview) {
                  const url = URL.createObjectURL(
                    new Blob([Buffer.from(content)]),
                  );
                  const element = createEmbedPreview(mimeType, url);
                  if (!element) {
                    URL.revokeObjectURL(url);
                    console.warn(
                      `Not able to provide a preview for this file.`,
                    );
                    return;
                  }
                  preview = { element, url };
                  embedPreviews.set(href, preview);
                }
                // The content may have changed while this was being fetched,
                // taking the link out of the document.
                if (e.isConnected) {
                  placeEmbedPreview(e, preview.element);
                }
              })
              .catch(console.error);
          }
        }
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

      // `{@html}` replaces the rendered tree on every change, taking the
      // emoji images with it, so they are put back on every render.
      twemoji(container, { exclude: ["21a9"] });

      if (!scrolledToHash && window.location.hash) {
        scrollIntoView(window.location.hash.substring(1));
      }
      scrolledToHash = true;
    });

    return () => {
      for (const icon of icons) {
        void unmount(icon);
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
    list-style-type: none;
    color: var(--color-text-secondary);
    /* A hanging indent: the box sits in the padding while a wrapped line lines
       up with the text above it. Together these are the box's own width. */
    padding-left: 1.75rem;
    text-indent: -1.75rem;
  }
  /* The indent is inherited, so block content inside an item has to undo it. */
  .markdown :global(li.task-item ul),
  .markdown :global(li.task-item ol),
  .markdown :global(li.task-item pre),
  .markdown :global(li.task-item blockquote),
  .markdown :global(li.task-item table) {
    text-indent: 0;
  }
  /* Without markers there is nothing to indent for, and `ul`'s user-agent top
     margin leaves a gap above every list. */
  .markdown :global(.task-list) {
    padding-left: 0;
    margin-top: 0;
  }
  .markdown :global(p.task-list-lead) {
    margin-bottom: 0;
  }
  /* Loose list items carry a paragraph, whose margin would space the items out
     more than a tight list's. */
  .markdown :global(li.task-item > p) {
    margin-bottom: 0;
  }
  .markdown :global(li.task-item .task-box) {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1.25rem;
    height: 1.25rem;
    margin-right: 0.5rem;
    /* `middle` centres the box on the x-height, which leaves it sitting low
       against the taller letters. Nudge it onto the cap-height centre; in `em`
       so it holds wherever the markdown is set at another size. */
    vertical-align: middle;
    position: relative;
    top: -0.09em;
    border: 1px solid var(--color-border-mid);
    border-radius: var(--border-radius-md);
    background-color: var(--color-surface-base);
  }
  .markdown :global(li.task-item .task-box.checked) {
    color: var(--color-text-brand);
  }
  .markdown :global(li.task-item .task-box:not(.checked) svg) {
    visibility: hidden;
  }
  .markdown :global(li.task-item button.task-box) {
    padding: 0;
    cursor: pointer;
    transition:
      background-color 0.1s ease,
      border-color 0.1s ease;
  }
  .markdown :global(li.task-item button.task-box:focus-visible) {
    outline: 2px solid var(--color-border-brand);
    outline-offset: 1px;
  }
  .markdown.busy :global(li.task-item button.task-box) {
    cursor: progress;
  }
  .markdown :global(li.task-item button.task-box:hover) {
    border-color: var(--color-border-strong);
    background-color: var(--color-surface-subtle);
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

<div
  class="markdown"
  class:busy={taskInFlight}
  aria-busy={taskInFlight}
  bind:this={container}>
  {@html renderMarkdown(doc.content, breaks)}
</div>
