import type { MarkedExtension, Tokens } from "marked";

import dompurify from "dompurify";
import escape from "lodash/escape.js";
import { Marked, Renderer as BaseRenderer } from "marked";
import { markedEmoji } from "marked-emoji";
import markedFootnote from "marked-footnote";
import katexMarkedExtension from "marked-katex-extension";
import markedLinkifyIt from "marked-linkify-it";

import emojis from "@app/lib/emojis";

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
  markedLinkifyIt({}, { fuzzyLink: false }),
  markedFootnote({ refMarkers: true }),
  markedEmoji({ emojis }),
  ((): MarkedExtension => ({
    extensions: [anchorMarkedExtension],
  }))(),
);
