import type { Config } from "@bindings/config/Config";

import type { RadReference } from "@app/lib/radUri";
import {
  explorerUrl,
  formatReference,
  isNodeId,
  isOid,
  isRepoId,
  issueType,
  parseExplorerUrl,
  parseReference,
  patchType,
} from "@app/lib/radUri";
import { explorerBase, explorerSeed } from "@app/lib/utils";

/**
 * A Radicle entity that can be referenced from a comment or description.
 *
 * References are stored as markdown links whose href is the entity's `rad:`
 * URI or DID, as decided in `docs/adr/0003-radicle-references.md`. The app
 * resolves the identifier on render and swaps the link for a rich chip.
 */
export type MentionTarget =
  | { type: "node"; nid: string }
  | { type: "repo"; rid: string }
  | { type: "cob"; kind: "issue" | "patch"; rid: string; oid: string }
  | { type: "commit"; rid: string; oid: string };

/**
 * The entity a reference points at, or `undefined` for a valid reference this
 * app has no chip for, such as a tree, a tag, an unknown COB type, or one
 * narrowed by a namespace, query or fragment a chip would drop.
 */
export function toMentionTarget(
  reference: RadReference,
): MentionTarget | undefined {
  if (reference.type === "did") return { type: "node", nid: reference.node };

  const { repo, resource, namespace, query, fragment } = reference.uri;
  if (namespace || query || fragment !== undefined) return undefined;
  const rid = `rad:${repo}`;
  if (!resource) return { type: "repo", rid };

  if (resource.type === "commit" && isOid(resource.ref)) {
    return { type: "commit", rid, oid: resource.ref };
  }
  if (resource.type === "cob" && resource.oid !== undefined) {
    if (resource.typeName === issueType) {
      return { type: "cob", kind: "issue", rid, oid: resource.oid };
    }
    if (resource.typeName === patchType) {
      return { type: "cob", kind: "patch", rid, oid: resource.oid };
    }
  }

  return undefined;
}

export function toRadReference(target: MentionTarget): RadReference {
  if (target.type === "node") return { type: "did", node: target.nid };

  const repo = target.rid.replace(/^rad:/, "");
  switch (target.type) {
    case "repo":
      return { type: "uri", uri: { repo } };
    case "commit":
      return {
        type: "uri",
        uri: { repo, resource: { type: "commit", ref: target.oid } },
      };
    case "cob":
      return {
        type: "uri",
        uri: {
          repo,
          resource: {
            type: "cob",
            typeName: target.kind === "issue" ? issueType : patchType,
            oid: target.oid,
          },
        },
      };
  }
}

/**
 * Parse a link href into the entity it refers to, or `undefined` when the
 * href is not a reference this app renders as a chip.
 *
 * Explorer URLs are accepted too, so a link pasted from the browser becomes
 * a chip that navigates in-app.
 */
export function parseMentionHref(href: string): MentionTarget | undefined {
  const trimmed = href.trim();
  const reference = parseReference(trimmed) ?? parseExplorerUrl(trimmed);

  return reference ? toMentionTarget(reference) : undefined;
}

/**
 * Parse an identifier written without its scheme, as copying a node or repo
 * id out of the UI produces.
 */
export function parseBareIdentifier(token: string): MentionTarget | undefined {
  if (isNodeId(token)) return { type: "node", nid: token };
  if (isRepoId(token)) return { type: "repo", rid: `rad:${token}` };

  return undefined;
}

const bareReferencePattern =
  /^(?:did:key:|rad:)[A-Za-z0-9._~!$&'()*+,;=:@/?#%-]+/;

/** Punctuation that may follow a reference in prose, as in `rad:z…'s`. */
const prosePunctuation = new Set(["'", '"', ",", ";", "!", "?", "(", ")", "*"]);

function isAlphanumeric(character: string): boolean {
  return /[0-9A-Za-z]/.test(character);
}

/**
 * The index of the first possible bare reference in `src` that starts a word,
 * used by the markdown tokenizer to skip ahead cheaply.
 */
export function bareReferenceStart(src: string): number | undefined {
  const match = /(^|[^0-9A-Za-z])(?:did:key:z|rad:)/.exec(src);

  return match ? match.index + match[1].length : undefined;
}

/**
 * Match a bare `rad:` URI or DID at the start of `src`, unless `previous`, the
 * character before it, makes it the tail of a word. Characters that cannot end
 * an identifier are left to the surrounding prose, so a reference at the end
 * of a sentence keeps its full stop outside the link.
 */
export function matchBareReference(
  src: string,
  previous?: string,
): { raw: string; reference: RadReference } | undefined {
  if (previous && isAlphanumeric(previous)) return undefined;

  const raw = bareReferencePattern.exec(src)?.[0];
  if (!raw) return undefined;

  let end = raw.length;
  for (;;) {
    while (end > 0 && !isAlphanumeric(raw[end - 1])) end--;
    const candidate = raw.slice(0, end);
    const reference = parseReference(candidate);
    if (reference) return { raw: candidate, reference };

    do end--;
    while (end > 0 && !prosePunctuation.has(raw[end]));
    if (end <= 0) return undefined;
  }
}

/** The canonical identifier of a reference, used to address and cache it. */
export function mentionHref(target: MentionTarget): string {
  return formatReference(toRadReference(target));
}

/**
 * The web explorer URL a reference points at, or `undefined` when the
 * explorer has no page for it.
 */
export function referenceUrl(
  reference: RadReference,
  config: Config,
): string | undefined {
  return explorerUrl(reference, explorerBase(config), explorerSeed(config));
}

export function mentionUrl(
  target: MentionTarget,
  config: Config,
): string | undefined {
  return referenceUrl(toRadReference(target), config);
}

/**
 * The markdown a reference is inserted as. The label is what clients that do
 * not resolve the identifier show, so it carries the alias, repo name or COB
 * title. Characters that would end a markdown link label are escaped.
 */
export function mentionMarkdown(target: MentionTarget, label: string): string {
  const escaped = label.replace(/[[\]\\]/g, character => `\\${character}`);

  return `[${escaped}](${mentionHref(target)})`;
}
