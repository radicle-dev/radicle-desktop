import { show } from "@app/lib/modal";
import { resolveRepoTarget } from "@app/lib/repoImages";

import UnopenableLink from "@app/modals/UnopenableLink.svelte";

export type DocumentLink =
  | { type: "fragment"; id: string }
  // A path in the repository, undefined for its root.
  | { type: "file"; path: string | undefined }
  | { type: "external" }
  | { type: "unsupported" };

// The schemes the shell plugin hands to the operating system when an anchor
// with `target="_blank"` is clicked.
const externalScheme = /^(?:https?|mailto|tel):/i;
const scheme = /^[a-z][a-z\d+.-]*:/i;

/**
 * Classify a link in the document at `from`. Links without a scheme are
 * paths in the repository, as they are on code forges, so they never point
 * at a page of the app.
 */
export function classifyDocumentLink(href: string, from: string): DocumentLink {
  if (href === "" || href.startsWith("#")) {
    return {
      type: "fragment",
      id: resolveRepoTarget(href, from)?.fragment ?? "",
    };
  }

  if (scheme.test(href)) {
    return externalScheme.test(href)
      ? { type: "external" }
      : { type: "unsupported" };
  }

  const target = resolveRepoTarget(href, from);
  if (!target) {
    return { type: "unsupported" };
  }

  const path = target.path.replace(/\/+$/, "");
  if (path === from && target.fragment !== undefined) {
    return { type: "fragment", id: target.fragment };
  }

  return { type: "file", path: path || undefined };
}

export interface DocumentLinkOptions {
  // Path of the document, which relative links are resolved against.
  path: string;
  // Opens a file of the repository the document belongs to. Without it,
  // links to files can't be followed.
  openFile?: (path: string | undefined) => void;
}

function scrollToFragment(id: string) {
  const target =
    document.getElementById(id) ?? document.getElementsByName(id)[0];
  target?.scrollIntoView();
}

/**
 * Make a link in a rendered document work inside the app. Following any link
 * that isn't external would otherwise load another page into the window and
 * restart the app. The document's options are read when the link is
 * followed, as the same markup can be shown for another file.
 */
export function enhanceDocumentLink(
  anchor: HTMLAnchorElement,
  options: () => DocumentLinkOptions,
) {
  const href = anchor.getAttribute("href");
  if (href === null) {
    return;
  }

  if (classifyDocumentLink(href, "").type === "external") {
    anchor.target = "_blank";
    anchor.rel = "noopener noreferrer";
    return;
  }

  anchor.onclick = event => {
    event.preventDefault();
    const { path, openFile } = options();
    const link = classifyDocumentLink(href, path);
    if (link.type === "fragment") {
      scrollToFragment(link.id);
    } else if (link.type === "file" && openFile) {
      openFile(link.path);
    } else {
      show({
        component: UnopenableLink,
        props: { link: href, reason: "unsupported" },
      });
    }
  };
}
