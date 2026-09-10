import type { Blob } from "@bindings/source/Blob";

import dompurify from "dompurify";

import { invoke } from "@app/lib/invoke";

// `markdown.ts` pins the shared DOMPurify singleton to the markdown allowlist
// via `setConfig`, which makes per-call configs no-ops. AsciiDoc output needs a
// wider allowlist, so it gets its own instance.
const purify = dompurify(window);

purify.setConfig({
  /* eslint-disable @typescript-eslint/naming-convention */
  ALLOWED_ATTR: [
    "alt",
    "class",
    "colspan",
    "data-lang",
    "href",
    "id",
    "rel",
    "rowspan",
    "scope",
    "src",
    "start",
    "target",
    "title",
    "type",
    "width",
  ],
  ALLOWED_TAGS: [
    "a",
    "b",
    "blockquote",
    "br",
    "caption",
    "cite",
    "code",
    "col",
    "colgroup",
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
    "i",
    "img",
    "kbd",
    "li",
    "mark",
    "ol",
    "p",
    "pre",
    "q",
    "span",
    "strong",
    "sub",
    "sup",
    "table",
    "tbody",
    "td",
    "th",
    "thead",
    "tr",
    "u",
    "ul",
  ],
  /* eslint-enable @typescript-eslint/naming-convention */
});

export interface AsciidocSource {
  rid: string;
  sha: string;
  path: string;
}

// Asciidoctor resolves include targets against `base_dir` and reads them with
// the Fetch API. Pointing `base_dir` at a synthetic origin and registering a
// cache for it turns every include into a repository blob lookup instead of a
// network request, while leaving the tag, line-range and nesting handling to
// Asciidoctor itself.
const includeOrigin = "https://include.radicle.invalid";

// The repository a conversion may read from, keyed by a token that only that
// conversion's `base_dir` carries. Keeping the repository out of the URL means
// a `..` target cannot reach a different repository or commit by rewriting the
// path: escaping past the token just drops the entry, and the read is refused.
const includeScopes = new Map<string, AsciidocSource>();

/**
 * Resolve a link written relative to the document at `from` into a repository
 * path. Returns undefined when the link leaves the repository, which covers
 * both other origins and targets that climb above the repository root.
 */
export function resolveRepoPath(
  href: string,
  from: string,
): string | undefined {
  const dir = from.split("/").slice(0, -1).join("/");

  let url: URL;
  try {
    // The trailing slash makes the base a directory, so `a.adoc` resolves as
    // a sibling of `from` rather than replacing its last segment.
    url = new URL(href, `${includeOrigin}/${dir ? `${dir}/` : ""}`);
  } catch {
    return undefined;
  }

  if (url.origin !== includeOrigin) {
    return undefined;
  }

  return decodeURIComponent(url.pathname).replace(/^\//, "") || undefined;
}

async function readInclude(uri: string): Promise<Response> {
  let url: URL;
  try {
    url = new URL(uri);
  } catch {
    return new Response(undefined, { status: 400 });
  }

  if (url.origin !== includeOrigin) {
    return new Response(undefined, { status: 403 });
  }

  const [, token, ...rest] = url.pathname.split("/").map(decodeURIComponent);
  const scope = includeScopes.get(token);
  const path = rest.join("/");
  if (!scope || !path) {
    return new Response(undefined, { status: 403 });
  }

  try {
    const blob = await invoke<Blob>("repo_blob", {
      rid: scope.rid,
      sha: scope.sha,
      path,
    });
    if (blob.binary) {
      return new Response(undefined, { status: 415 });
    }

    return new Response(blob.content);
  } catch {
    return new Response(undefined, { status: 404 });
  }
}

let asciidoctor: Promise<typeof import("@asciidoctor/core")> | undefined;

function loadAsciidoctor() {
  if (!asciidoctor) {
    asciidoctor = import("@asciidoctor/core").then(module => {
      const cache = new module.HttpCache();
      cache.read = readInclude;
      module.HttpCacheManager.setCache(cache);

      return module;
    });
  }

  return asciidoctor;
}

export async function renderAsciidoc(
  content: string,
  source: AsciidocSource,
): Promise<string> {
  const { convert } = await loadAsciidoctor();

  const token = crypto.randomUUID();
  const dir = source.path.split("/").slice(0, -1).join("/");
  includeScopes.set(token, source);

  let html;
  try {
    html = await convert(content, {
      // `server` keeps docinfo and absolute paths disabled while allowing
      // includes; every read they perform goes through `readInclude`.
      safe: "server",
      standalone: false,
      /* eslint-disable @typescript-eslint/naming-convention */
      to_file: false,
      base_dir: [includeOrigin, token, dir].filter(Boolean).join("/"),
      attributes: {
        showtitle: true,
        "allow-uri-read": "",
        "cache-uri": "",
      },
      /* eslint-enable @typescript-eslint/naming-convention */
    });
  } finally {
    includeScopes.delete(token);
  }

  if (typeof html !== "string") {
    throw new Error("Asciidoctor did not return a converted document");
  }

  return purify.sanitize(html);
}
