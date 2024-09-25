import type { MarkedExtension, Tokens } from "marked";

import dompurify from "dompurify";
import katexMarkedExtension from "marked-katex-extension";
import markedFootnote from "marked-footnote";
import markedLinkifyIt from "marked-linkify-it";
import { Marked, Renderer as BaseRenderer } from "marked";
import { markedEmoji } from "marked-emoji";

import emojis from "@app/lib/emojis";

dompurify.setConfig({
  // eslint-disable-next-line @typescript-eslint/naming-convention
  SANITIZE_DOM: false,
  // eslint-disable-next-line @typescript-eslint/naming-convention
  FORBID_TAGS: ["textarea", "style"],
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
    const escapedText = text
      // By lowercasing we avoid casing mismatches, between headings and links.
      .toLowerCase()
      .replace(/[^\w]+/g, "-")
      .replace(/^-|-$/g, "");

    return `<h${depth} id="${escapedText}">${text}</h${depth}>`;
  }

  link({ href, title, tokens }: Tokens.Link): string {
    const text = this.parser.parseInline(tokens);
    if (href.startsWith("#")) {
      // By lowercasing we avoid casing mismatches, between headings and links.
      return `<a ${title ? `title="${title}"` : ""} href="${href.toLowerCase()}">${text}</a>`;
    }

    return `<a ${title ? `title="${title}"` : ""} href="${href}">${text}</a>`;
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
