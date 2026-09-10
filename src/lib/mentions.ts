import type { Config } from "@bindings/config/Config";

import { explorerUrl, parseNodeId, parseRepositoryId } from "@app/lib/utils";

/**
 * A Radicle entity that can be referenced from a comment or description.
 *
 * References are stored in the underlying data model as ordinary markdown
 * links whose href is a Radicle identifier, so any markdown reader still
 * shows the human readable label. The app resolves the identifier on render
 * and swaps the link for a rich chip.
 */
export type MentionTarget =
  | { type: "node"; nid: string }
  | { type: "repo"; rid: string }
  | { type: "cob"; kind: "issue" | "patch"; rid: string; oid: string }
  | { type: "commit"; rid: string; oid: string };

/**
 * Matches a bare DID anywhere in a run of text, e.g. `did:key:z6Mk…`.
 *
 * Base58 excludes `0`, `O`, `I` and `l`, but the character class is kept
 * permissive and the payload validated with `parseNodeId`, so a near-miss is
 * left as plain text rather than becoming a broken link.
 */
const didPattern = /did:key:z[1-9A-HJ-NP-Za-km-z]+/;

/** Matches a bare RID, optionally followed by an object path. */
const ridPattern =
  /rad:z[1-9A-HJ-NP-Za-km-z]+(?:\/(?:issues|patches|commits)\/[0-9a-f]{6,40})?/;

/** A COB object id: a full or abbreviated git oid. */
const oidPattern = /^[0-9a-f]{6,40}$/;

/**
 * A full git oid, as copying an issue, patch or commit id out of the UI
 * produces. Only the full form is recognised without a scheme, since a short
 * hex run is too easily an ordinary word.
 */
export const bareOidPattern = /^[0-9a-f]{40}$/;

/**
 * An identifier written without its scheme. A repo id decodes to 20 bytes and
 * a node id to 34, so both are far longer than any word that merely starts
 * with `z`; the length floor is what makes the check cheap and safe.
 */
const bareIdentifierPattern = /^z[1-9A-HJ-NP-Za-km-z]{20,}$/;

/** Matches a bare reference only at the very start of the input. */
const anchoredReferencePattern = new RegExp(
  `^(?:${didPattern.source}|${ridPattern.source})`,
);

/**
 * The index of the first possible bare reference in `src`, used by the
 * markdown tokenizer to skip ahead cheaply.
 */
export function bareReferenceStart(src: string): number | undefined {
  return src.match(/did:key:z|rad:z/)?.index;
}

/**
 * Match a bare reference at the start of `src`, returning the raw text and the
 * entity it points at. Trailing punctuation is not consumed, so a reference at
 * the end of a sentence keeps its full stop outside the link.
 */
export function matchBareReference(
  src: string,
): { raw: string; target: MentionTarget } | undefined {
  const match = anchoredReferencePattern.exec(src);
  if (!match) return undefined;

  let raw = match[0];
  // A COB path can only end in hex, and base58 has no trailing punctuation,
  // so anything else at the tail belongs to the surrounding prose.
  while (raw.length > 0 && !/[0-9A-Za-z]$/.test(raw)) {
    raw = raw.slice(0, -1);
  }

  const target = parseMentionHref(raw);

  return target ? { raw, target } : undefined;
}

/**
 * Parse a link href into the entity it refers to, or `undefined` when the
 * href is not a Radicle reference.
 *
 * Accepts the `did:key:` and `rad:` forms the app writes itself, and web
 * explorer URLs, so a link pasted from the browser becomes a chip that
 * navigates in-app instead of leaving the desktop app.
 */
export function parseMentionHref(href: string): MentionTarget | undefined {
  const trimmed = href.trim();

  if (trimmed.startsWith("did:key:")) {
    const parsed = parseNodeId(trimmed);
    return parsed ? { type: "node", nid: parsed.pubkey } : undefined;
  }
  if (trimmed.startsWith("rad:")) {
    return parseRadReference(trimmed);
  }
  return parseExplorerHref(trimmed);
}

/** Parse `rad:z…`, `rad:z…/issues/<oid>` and `rad:z…/patches/<oid>`. */
function parseRadReference(reference: string): MentionTarget | undefined {
  const [rid, kind, oid, ...rest] = reference.split("/");
  const parsed = parseRepositoryId(rid);
  if (!parsed || rest.length > 0) return undefined;

  if (kind === undefined && oid === undefined) {
    return { type: "repo", rid: `rad:${parsed.pubkey}` };
  }
  if (oid === undefined || !oidPattern.test(oid)) return undefined;

  if (kind === "commits") {
    return { type: "commit", rid: `rad:${parsed.pubkey}`, oid };
  }
  if (kind !== "issues" && kind !== "patches") return undefined;

  return {
    type: "cob",
    kind: kind === "issues" ? "issue" : "patch",
    rid: `rad:${parsed.pubkey}`,
    oid,
  };
}

/**
 * Parse an identifier written without its scheme, as copying a node or repo id
 * out of the UI produces.
 *
 * A node id and a repo id are both base58 strings beginning with `z`, but
 * decode to different lengths, so the parsers tell them apart. Anything that
 * is not a well-formed identifier is rejected, which is what keeps an ordinary
 * word beginning with `z` from being treated as one.
 */
export function parseBareIdentifier(token: string): MentionTarget | undefined {
  // Checked before parsing for two reasons: an ordinary word starting with `z`
  // is never this long, and `parseNodeId` logs to the console when bs58
  // decoding fails, which would fire on every keystroke of a word like `zlib`.
  if (!bareIdentifierPattern.test(token)) return undefined;

  const nid = parseNodeId(token);
  if (nid) return { type: "node", nid: nid.pubkey };

  const rid = parseRepositoryId(token);
  if (rid) return { type: "repo", rid: `rad:${rid.pubkey}` };

  return undefined;
}

/**
 * Parse a web explorer URL, e.g.
 * `https://app.radicle.xyz/nodes/<seed>/rad:z…/issues/<oid>` or
 * `https://app.radicle.xyz/nodes/<seed>/users/did:key:z…`.
 *
 * The seed host and any node prefix are ignored: what matters is the entity
 * the path points at, which is location independent.
 */
function parseExplorerHref(href: string): MentionTarget | undefined {
  let url: URL;
  try {
    url = new URL(href);
  } catch {
    return undefined;
  }
  if (url.protocol !== "https:" && url.protocol !== "http:") return undefined;

  // Copying a URL out of a browser percent-encodes the colons in a DID or a
  // RID, so segments are decoded before anything is recognised in them.
  const segments = url.pathname
    .split("/")
    .filter(segment => segment !== "")
    .map(decodeSegment);

  const userIndex = segments.indexOf("users");
  if (userIndex !== -1) {
    const parsed = parseNodeId(segments[userIndex + 1] ?? "");
    return parsed ? { type: "node", nid: parsed.pubkey } : undefined;
  }

  const ridIndex = segments.findIndex(segment => segment.startsWith("rad:z"));
  if (ridIndex === -1) return undefined;

  return parseRadReference(segments.slice(ridIndex).join("/"));
}

/** Decode a path segment, leaving a malformed escape sequence as it stands. */
function decodeSegment(segment: string): string {
  try {
    return decodeURIComponent(segment);
  } catch {
    return segment;
  }
}

/** The identifier a reference resolves to, used to address and cache it. */
export function mentionHref(target: MentionTarget): string {
  switch (target.type) {
    case "node":
      return `did:key:${target.nid}`;
    case "repo":
      return target.rid;
    case "cob":
      return `${target.rid}/${target.kind === "issue" ? "issues" : "patches"}/${target.oid}`;
    case "commit":
      return `${target.rid}/commits/${target.oid}`;
  }
}

/**
 * The web explorer URL a reference points at.
 *
 * A person lives under `users/`, everything else under its repo id, which is
 * what the identifier already reads as.
 */
export function mentionUrl(target: MentionTarget, config: Config): string {
  const path =
    target.type === "node"
      ? `users/${mentionHref(target)}`
      : mentionHref(target);

  return explorerUrl(path, config);
}

/**
 * The markdown a reference is inserted as. The label is what other clients
 * show, so it carries the alias, repo name or COB title.
 *
 * The destination is the explorer URL rather than the bare identifier: a
 * `rad:` or `did:key:` href means nothing to another client, whose sanitizer
 * drops the unknown scheme and leaves nothing to click. A web URL still opens
 * the right page in a browser, and this app recognises it on render and turns
 * it back into a chip that navigates in-app.
 *
 * Characters that would terminate a markdown link label or destination are
 * escaped, so a title containing brackets cannot break out of the link.
 */
export function mentionMarkdown(
  target: MentionTarget,
  label: string,
  config: Config | undefined,
): string {
  const escaped = label.replace(/[[\]\\]/g, character => `\\${character}`);
  // Without config there is no explorer to point at, so the identifier is
  // written instead: still a working reference in this app, just not elsewhere.
  const destination = config ? mentionUrl(target, config) : mentionHref(target);

  return `[${escaped}](${destination})`;
}
