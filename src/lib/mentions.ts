import type Icon from "@app/components/Icon.svelte";
import type { RepoRoute } from "@app/views/repo/router";
import type { Config } from "@bindings/config/Config";
import type { ComponentProps } from "svelte";

import type { RadReference } from "@app/lib/radUri";
import {
  cobListType,
  filePath,
  formatReference,
  isNodeId,
  isOid,
  isRepoId,
  issueType,
  parseExplorerUrl,
  parseReference,
  patchType,
  releaseType,
} from "@app/lib/radUri";
import { explorerLink, formatOid, truncateId } from "@app/lib/utils";

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

/** Longest bare reference linked in prose, generous for a file path. */
const maximumReferenceLength = 512;

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
 * Match a bare `rad:` URI or DID at the start of `src`. Characters that cannot
 * end an identifier are left to the surrounding prose, so a reference at the
 * end of a sentence keeps its full stop outside the link.
 */
export function matchBareReference(
  src: string,
): { raw: string; reference: RadReference } | undefined {
  const raw = bareReferencePattern.exec(src)?.[0];
  // Each retry below parses again, so an unbounded run of punctuation in a
  // comment would take quadratic time to render.
  if (!raw || raw.length > maximumReferenceLength) return undefined;

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

/** The in-app page for a reference, or `undefined` when the app has none. */
export function referenceRoute(reference: RadReference): RepoRoute | undefined {
  if (reference.type !== "uri") return undefined;
  const { repo, namespace, resource } = reference.uri;
  const rid = `rad:${repo}`;

  const file = filePath(reference);
  if (file) {
    return {
      resource: "repo.home",
      rid,
      peer: namespace,
      revision: shortRevision(file.revision),
      path: file.path || undefined,
    };
  }

  if (!resource) return { resource: "repo.home", rid, peer: namespace };
  if (resource.type === "commit" && isOid(resource.ref)) {
    return { resource: "repo.commit", rid, commit: resource.ref };
  }
  if (resource.type === "cob" && resource.oid !== undefined) {
    if (resource.typeName === issueType) {
      return {
        resource: "repo.issue",
        rid,
        issue: resource.oid,
        status: "all",
      };
    }
    if (resource.typeName === patchType) {
      return {
        resource: "repo.patch",
        rid,
        patch: resource.oid,
        status: undefined,
        reviewId: undefined,
      };
    }
  }
  if (resource.type === "commit" || resource.type === "tag") {
    return {
      resource: "repo.home",
      rid,
      peer: namespace,
      revision: shortRevision(resource.ref),
    };
  }

  const typeName = cobListType(reference);
  if (typeName === issueType) {
    return { resource: "repo.issues", rid, status: "all" };
  }
  if (typeName === patchType) {
    return { resource: "repo.patches", rid, status: undefined };
  }

  return undefined;
}

export interface LinkDescription {
  /** Written into the markdown link's label, e.g. `heartwood: src/lib.rs`. */
  label: string;
  primary: string;
  secondary: string;
  icon: ComponentProps<typeof Icon>["name"];
}

const cobLists: Record<
  string,
  { label: string; icon: LinkDescription["icon"] }
> = {
  [issueType]: { label: "Issues", icon: "issue" },
  [patchType]: { label: "Patches", icon: "patch" },
  [releaseType]: { label: "Releases", icon: "archive" },
};

/**
 * How a reference without a chip is labelled and drawn, or `undefined` for
 * one that is not offered as a link. `repoName` names the repo it is in, and
 * `peerName` the remote, when it is one.
 */
export function describeLink(
  reference: RadReference,
  repoName: string,
  peerName?: string,
): LinkDescription | undefined {
  if (reference.type !== "uri") return undefined;
  const { resource, namespace, fragment } = reference.uri;

  const file = filePath(reference);
  if (file) {
    const revision = shortRevision(file.revision);
    const shown = isOid(revision) ? formatOid(revision) : revision;
    if (!file.path) {
      return {
        label: `${repoName}: ${shown}`,
        primary: shown,
        secondary: repoName,
        icon: "folder",
      };
    }
    // A line is named the way the explorer anchors it, e.g. `#L10`.
    const line = fragment ? `#${fragment}` : "";
    return {
      label: `${repoName}: ${file.path}${line}`,
      primary: `${file.path}${line}`,
      secondary: shown,
      icon: "document",
    };
  }

  const typeName = cobListType(reference);
  const list = typeName ? cobLists[typeName] : undefined;
  if (list) {
    return {
      label: `${repoName}: ${list.label.toLowerCase()}`,
      primary: list.label,
      secondary: repoName,
      icon: list.icon,
    };
  }

  if (
    resource?.type === "cob" &&
    resource.typeName === releaseType &&
    resource.oid !== undefined
  ) {
    const release = `release ${formatOid(resource.oid)}`;
    return {
      label: `${repoName}: ${release}`,
      primary: `Release ${formatOid(resource.oid)}`,
      secondary: repoName,
      icon: "archive",
    };
  }

  if (
    (resource?.type === "commit" && !isOid(resource.ref)) ||
    resource?.type === "tag"
  ) {
    const revision = shortRevision(resource.ref);
    return {
      label: `${repoName}: ${revision}`,
      primary: revision,
      secondary: repoName,
      icon: resource.type === "tag" ? "label" : "branch",
    };
  }

  if (!resource && namespace) {
    const peer = peerName ?? truncateId(namespace);
    return {
      label: `${repoName}: ${peer}`,
      primary: peer,
      secondary: repoName,
      icon: "repository",
    };
  }

  return undefined;
}

/** A branch or tag as the app's routes name it, without `refs/heads/`. */
function shortRevision(ref: string): string {
  return ref.replace(/^refs\/(?:heads|tags)\//, "");
}

/** The canonical identifier of a reference, used to address and cache it. */
export function mentionHref(target: MentionTarget): string {
  return formatReference(toRadReference(target));
}

/** The web explorer URL an entity points at. */
export function mentionUrl(
  target: MentionTarget,
  config: Config,
): string | undefined {
  return explorerLink(toRadReference(target), config);
}

/**
 * The markdown a reference is inserted as. The label is what clients that do
 * not resolve the identifier show, so it carries the alias, repo name or COB
 * title. Characters that would end a markdown link label are escaped.
 */
export function mentionMarkdown(target: MentionTarget, label: string): string {
  return referenceMarkdown(toRadReference(target), label);
}

export function referenceMarkdown(
  reference: RadReference,
  label: string,
): string {
  const escaped = label.replace(/[[\]\\]/g, character => `\\${character}`);

  return `[${escaped}](${formatReference(reference)})`;
}
