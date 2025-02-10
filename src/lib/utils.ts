import type { ComponentProps } from "svelte";

import type { Author } from "@bindings/cob/Author";
import type { Issue } from "@bindings/cob/issue/Issue";
import type { Patch } from "@bindings/cob/patch/Patch";
import type { Review } from "@bindings/cob/patch/Review";

import bs58 from "bs58";
import twemojiModule from "twemoji";
import md5 from "md5";

import NodeId from "@app/components/NodeId.svelte";

export const unreachable = (value: never): never => {
  throw new Error(`Unreachable code: ${value}`);
};

export function formatRepositoryId(id: string): string {
  const parsedId = parseRepositoryId(id);

  if (parsedId) {
    return `${parsedId.prefix}${truncateId(parsedId.pubkey)}`;
  }

  return id;
}

export function parseRepositoryId(
  rid: string,
): { prefix: string; pubkey: string } | undefined {
  const match = /^(rad:)?(z[a-zA-Z0-9]+)$/.exec(rid);
  if (match) {
    const hex = bs58.decode(match[2].substring(1));
    if (hex.byteLength !== 20) {
      return undefined;
    }

    return { prefix: match[1] || "rad:", pubkey: match[2] };
  }

  return undefined;
}

export function truncateId(pubkey: string): string {
  return `${pubkey.substring(0, 6)}…${pubkey.slice(-6)}`;
}

export function truncateDid(did: string): string {
  return `did:key:${truncateId(publicKeyFromDid(did))}`;
}

export function didFromPublicKey(publicKey: string) {
  return `did:key:${publicKey}`;
}

export function publicKeyFromDid(did: string) {
  return did.replace("did:key:", "");
}

export function isCommit(input: string): boolean {
  return /^[a-f0-9]{40}$/.test(input);
}

export function formatOid(id: string): string {
  return id.substring(0, 7);
}

export const formatTimestamp = (
  timestamp: number,
  current = new Date().getTime(),
): string => {
  const units: Record<string, number> = {
    year: 24 * 60 * 60 * 1000 * 365,
    month: (24 * 60 * 60 * 1000 * 365) / 12,
    day: 24 * 60 * 60 * 1000,
    hour: 60 * 60 * 1000,
    minute: 60 * 1000,
    second: 1000,
  };

  const rtf = new Intl.RelativeTimeFormat("en", {
    numeric: "auto",
    style: "long",
  });
  const elapsed = current - timestamp;

  if (elapsed > units["year"]) {
    return "more than a year ago";
  } else if (elapsed < 0) {
    return "now"; // If elapsed is a negative number we are dealing with an item from the future, and we return "now"
  }

  for (const u in units) {
    if (elapsed > units[u] || u === "second") {
      // We convert the division result to a negative number to get "XX [unit] ago"
      return rtf.format(
        Math.round(elapsed / units[u]) * -1,
        u as Intl.RelativeTimeFormatUnit,
      );
    }
  }

  return new Date(timestamp).toUTCString();
};

// Svelte action for replacing emojis within an element with Twemoji SVGs.
// This action is non-reactive; it only runs when the element is mounted.
//
// Usage: <span use:twemoji>👍</span>
export function twemoji(
  node: HTMLElement,
  { exclude }: { exclude: string[] } = { exclude: [] },
) {
  twemojiModule.parse(node, {
    callback: (icon, options) => {
      const { base, size, ext } = options as Record<string, string>;
      if (!exclude.includes(icon)) {
        return `${base}${size}/${icon}${ext}`;
      }
      return false;
    },
    base: "/",
    folder: "twemoji",
    ext: ".svg",
    className: `txt-emoji`,
  });
}

// Converts a single emoji character to an <img> tag using Twemoji SVG assets.
// This function is useful in reactive contexts where Svelte actions won't
// re-run automatically.
//
// Usage: <span>{@html emojiToTwemoji("👍")}</span>
export function emojiToTwemoji(emoji: string, exclude?: string[]) {
  const filename = emoji.codePointAt(0)?.toString(16);
  if (!filename || exclude?.includes(filename)) {
    return "";
  }
  return `<img alt="${emoji}" src="/twemoji/${filename}.svg" class="txt-emoji">`;
}

export function scrollIntoView(id: string, options?: ScrollIntoViewOptions) {
  const lineElement = document.getElementById(id);
  if (lineElement) {
    lineElement.scrollIntoView(options);
  }
}

export const issueStatusColor: Record<Issue["state"]["status"], string> = {
  open: "var(--color-fill-success)",
  closed: "var(--color-foreground-red)",
};

export const issueStatusBackgroundColor: Record<
  Issue["state"]["status"],
  string
> = {
  open: "var(--color-fill-diff-green)",
  closed: "var(--color-fill-diff-red)",
};

export const patchStatusColor: Record<Patch["state"]["status"], string> = {
  draft: "var(--color-fill-gray)",
  open: "var(--color-fill-success)",
  archived: "var(--color-foreground-yellow)",
  merged: "var(--color-fill-primary)",
};

export const patchStatusBackgroundColor: Record<
  Patch["state"]["status"],
  string
> = {
  draft: "var(--color-fill-ghost)",
  open: "var(--color-fill-diff-green)",
  archived: "var(--color-fill-private)",
  merged: "var(--color-fill-delegate)",
};

export function authorForNodeId(author: Author): ComponentProps<typeof NodeId> {
  return { publicKey: publicKeyFromDid(author.did), alias: author.alias };
}

export function absoluteTimestamp(timestamp: number) {
  return new Date(Number(timestamp)).toLocaleString();
}

export function formatEditedCaption(author: Author, timestamp: number) {
  return `${author.alias ? author.alias : truncateDid(author.did)} edited ${absoluteTimestamp(timestamp)}`;
}

export function pluralize(singular: string, count: number): string {
  return count === 1 ? singular : `${singular}s`;
}

export function isMac() {
  if (
    (navigator.platform && navigator.platform.includes("Mac")) ||
    navigator.userAgent.includes("OS X")
  ) {
    return true;
  } else {
    return false;
  }
}

export function modifierKey() {
  return isMac() ? "⌘" : "ctrl";
}

export function parseNodeId(
  nid: string,
): { prefix: string; pubkey: string } | undefined {
  const match = /^(did:key:)?(z[a-zA-Z0-9]+)$/.exec(nid);
  if (match) {
    let hex: Uint8Array | undefined = undefined;
    try {
      hex = bs58.decode(match[2].substring(1));
    } catch (error) {
      console.error("utils.parseNodId: Not able to decode received NID", error);
      return undefined;
    }
    // This checks also that the first 2 bytes are equal
    // to the ed25519 public key type used.
    if (hex && !(hex.byteLength === 34 && hex[0] === 0xed && hex[1] === 1)) {
      return undefined;
    }

    return { prefix: match[1] || "did:key:", pubkey: match[2] };
  }

  return undefined;
}

// Get the gravatar URL of an email.
export function gravatarURL(email: string): string {
  const address = email.trim().toLowerCase();
  const hash = md5(address);

  return `https://www.gravatar.com/avatar/${hash}`;
}

export function verdictIcon(verdict: Review["verdict"]) {
  if (verdict === "accept") {
    return "comment-checkmark";
  } else if (verdict === "reject") {
    return "comment-cross";
  } else {
    return "comment";
  }
}

export function verdictIconColor(verdict: Review["verdict"]) {
  if (verdict === "accept") {
    return "var(--color-foreground-success)";
  } else if (verdict === "reject") {
    return "var(--color-foreground-red)";
  } else {
    return "var(--color-foreground-dim)";
  }
}
