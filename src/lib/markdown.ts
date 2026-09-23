import type { MarkedExtension, Tokens } from "marked";

import dompurify from "dompurify";
import escape from "lodash/escape.js";
import { Marked, Renderer as BaseRenderer } from "marked";
import { markedEmoji } from "marked-emoji";
import markedFootnote from "marked-footnote";
import katexMarkedExtension from "marked-katex-extension";
import markedLinkifyIt from "marked-linkify-it";

import emojis from "@app/lib/emojis";
import { parseFrontmatter } from "@app/lib/frontmatter";

dompurify.setConfig({
  /* eslint-disable @typescript-eslint/naming-convention */
  ALLOWED_ATTR: [
    "align",
    "checked",
    "class",
    "href",
    "id",
    "name",
    "rel",
    "target",
    "text",
    "title",
    "src",
    "type",
  ],
  ALLOWED_TAGS: [
    "a",
    "blockquote",
    "br",
    "code",
    "dd",
    "del",
    "div",
    "dl",
    "dt",
    "em",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "hr",
    "img",
    "input",
    "li",
    "ol",
    "p",
    "pre",
    "strong",
    "table",
    "tbody",
    "td",
    "th",
    "thead",
    "tr",
    "ul",
  ],
  /* eslint-enable @typescript-eslint/naming-convention */
});

// Converts self closing anchor tags into empty anchor tags, to avoid erratic wrapping behaviour
// e.g. <a name="test"/> -> <a name="test"></a>
const anchorMarkedExtension = {
  name: "sanitizedAnchor",
  level: "block",
  start: (src: string) => src.match(/<a name="([\w]+)"\/>/)?.index,
  tokenizer(src: string) {
    const match = src.match(/^<a name="([\w]+)"\/>/);
    if (match) {
      return {
        type: "sanitizedAnchor",
        raw: match[0],
        text: match[1].trim(),
      };
    }
  },
  renderer: (token: Tokens.Generic): string => `<a name="${token.text}"></a>`,
};

// Marks the checkboxes of task-list items, so they can be told apart from any
// checkbox written as raw HTML.
export const TASK_CHECKBOX_CLASS = "task-checkbox";

export class Renderer extends BaseRenderer {
  /**
   * If `baseUrl` is provided, all hrefs attributes in anchor tags, except those
   * starting with `#`, are resolved with respect to `baseUrl`
   */
  constructor() {
    super();
  }
  // Overwrites the rendering of heading tokens.
  // Since there are possible non ASCII characters in headings,
  // we escape them by replacing them with dashes and,
  // trim eventual dashes on each side of the string.
  heading({ tokens, depth }: Tokens.Heading) {
    const text = this.parser.parseInline(tokens);
    const id = text
      // By lowercasing we avoid casing mismatches, between headings and links.
      .toLowerCase()
      .replace(/[^\w]+/g, "-")
      .replace(/^-|-$/g, "");

    return `<h${depth} id="${id}">${text}</h${depth}>`;
  }

  checkbox({ checked }: Tokens.Checkbox): string {
    return `<input class="${TASK_CHECKBOX_CLASS}" ${checked ? 'checked="" ' : ""}disabled="" type="checkbox"> `;
  }

  link({ href, title, tokens }: Tokens.Link): string {
    const text = this.parser.parseInline(tokens);
    const normalizedHref = href.startsWith("#") ? href.toLowerCase() : href;

    const attrs = [`href="${escape(normalizedHref)}"`];
    if (title) {
      attrs.push(`title="${escape(title)}"`);
    }

    return `<a ${attrs.join(" ")}>${text}</a>`;
  }
}

export default new Marked();

export const markdownWithExtensions = new Marked(
  katexMarkedExtension({ throwOnError: false }),
  markedLinkifyIt({ fuzzyLink: false }),
  markedFootnote({ refMarkers: true }),
  markedEmoji({ emojis }),
  ((): MarkedExtension => ({
    extensions: [anchorMarkedExtension],
  }))(),
);

/**
 * Renders markdown to sanitized HTML. Everything that has to agree with what
 * is on screen goes through this, so it is the single source of the output.
 */
export function renderMarkdown(content: string, breaks = false): string {
  return dompurify.sanitize(
    markdownWithExtensions.parse(content, {
      renderer: new Renderer(),
      breaks,
    }) as string,
  );
}

// Anything that could be the box of a task-list item: a list marker, spacing,
// then the box. It over-matches on purpose, since every candidate is checked
// against the rendered output before it is used.
const TASK_CANDIDATE_RE = /(?:[-*+]|\d{1,9}[.)])[ \t]+\[([ xX])\]/g;
const CODE_FENCE_RE = /^[ \t]*(`{3,}|~{3,})/;

/**
 * Whether a rendered checkbox came from task-list syntax. Both the rendered
 * page and `toggleTask` count task items with this, so the two always agree
 * on which box is which.
 */
export function isTaskCheckbox(element: Element): boolean {
  return element.getAttribute("class") === TASK_CHECKBOX_CLASS;
}

function taskStates(content: string): boolean[] {
  // A template's content is inert, so nothing in it loads or runs.
  const template = document.createElement("template");
  template.innerHTML = renderMarkdown(content);
  return Array.from(template.content.querySelectorAll('input[type="checkbox"]'))
    .filter(isTaskCheckbox)
    .map(input => input.hasAttribute("checked"));
}

// The character ranges of fenced code blocks. Only used to decide which
// candidates to try first, so it does not have to be exact.
function fencedRanges(content: string): [number, number][] {
  const ranges: [number, number][] = [];
  let open: { fence: string; start: number } | undefined;
  let offset = 0;
  for (const line of content.split("\n")) {
    const fence = CODE_FENCE_RE.exec(line)?.[1];
    if (fence && open === undefined) {
      open = { fence, start: offset };
    } else if (
      fence &&
      open &&
      fence[0] === open.fence[0] &&
      fence.length >= open.fence.length
    ) {
      ranges.push([open.start, offset + line.length]);
      open = undefined;
    }
    offset += line.length + 1;
  }
  if (open) {
    ranges.push([open.start, content.length]);
  }
  return ranges;
}

/**
 * Flips the checkbox of the task-list item at `index`, counting task items in
 * the order they are rendered. Returns the whole document with that one box
 * rewritten, or `undefined` when there is no task item at that index.
 *
 * Code blocks, HTML blocks and comments can all contain text that looks like a
 * task item without rendering as one, so a match in the source is only taken
 * once re-rendering shows that it flips exactly the requested box.
 */
export function toggleTask(input: string, index: number): string | undefined {
  // Frontmatter is not rendered, so it is left out of the search.
  const { content } = parseFrontmatter(input);
  const prefix = input.slice(0, input.length - content.length);
  const states = taskStates(content);
  if (index < 0 || index >= states.length) {
    return undefined;
  }

  const positions = Array.from(
    content.matchAll(TASK_CANDIDATE_RE),
    match => match.index + match[0].length - 2,
  );
  // Each candidate costs a render, so the likely ones go first: outside code
  // blocks, starting with the one in the same position as the box.
  const fenced = fencedRanges(content);
  const inCode = (p: number) =>
    fenced.some(([start, end]) => p >= start && p <= end);
  const outside = positions.filter(p => !inCode(p));
  const likely = outside[index];
  const ordered = [
    ...(likely === undefined ? [] : [likely]),
    ...outside.filter(p => p !== likely),
    ...positions.filter(inCode),
  ];

  for (const position of ordered) {
    const next =
      content.slice(0, position) +
      (content[position] === " " ? "x" : " ") +
      content.slice(position + 1);
    const nextStates = taskStates(next);
    if (
      nextStates.length === states.length &&
      nextStates.every((checked, i) =>
        i === index ? checked !== states[i] : checked === states[i],
      )
    ) {
      return prefix + next;
    }
  }

  return undefined;
}
