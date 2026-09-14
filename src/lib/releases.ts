import type { Author } from "@bindings/cob/Author";
import type { Artifact } from "@bindings/cob/release/Artifact";
import type { Location } from "@bindings/cob/release/Location";
import type { ReleaseCounts } from "@bindings/cob/release/ReleaseCounts";
import type { ReleaseScope } from "@bindings/cob/release/ReleaseScope";
import type { Commit } from "@bindings/repo/Commit";
import type { RepoRefs } from "@bindings/repo/RepoRefs";
import type { Tag } from "@bindings/repo/Tag";

// With no scope asked for and no delegate releases, open on the untrusted
// scope rather than on an empty list.
export function releaseListScope(
  requested: ReleaseScope | undefined,
  counts: ReleaseCounts,
): ReleaseScope {
  if (requested !== undefined) {
    return requested;
  }
  return counts.delegate === 0 && counts.other > 0 ? "untrusted" : "trusted";
}

export interface ArtifactView {
  scope: ReleaseScope;
  shown: Artifact[];
  // Redacted artifacts within the current author scope.
  redactedCount: number;
  // What each scope would show, so an artifact hidden as redacted never
  // counts towards a scope.
  counts: Record<ReleaseScope, number>;
  // Filter only when both scopes hold something.
  showFilters: boolean;
}

// Split a release's artifacts into two disjoint scopes by author. Show the
// scope asked for unless it is empty, and hide artifacts redacted by their
// author or a delegate unless asked to show them.
export function artifactView(
  artifacts: Artifact[],
  delegates: Set<string>,
  wanted: ReleaseScope,
  showRedacted: boolean,
): ArtifactView {
  const byScope: Record<ReleaseScope, Artifact[]> = {
    trusted: artifacts.filter(a => delegates.has(a.author.did)),
    untrusted: artifacts.filter(a => !delegates.has(a.author.did)),
  };
  const visible = (list: Artifact[]) =>
    showRedacted ? list : list.filter(a => !a.redacted);
  const other = wanted === "trusted" ? "untrusted" : "trusted";
  const scope = byScope[wanted].length > 0 ? wanted : other;

  return {
    scope,
    shown: visible(byScope[scope]),
    redactedCount: byScope[scope].filter(a => a.redacted).length,
    counts: {
      trusted: visible(byScope.trusted).length,
      untrusted: visible(byScope.untrusted).length,
    },
    showFilters: byScope.trusted.length > 0 && byScope.untrusted.length > 0,
  };
}

// Delegate takes precedence over author, since the author may also be a
// delegate.
export function redactedByDelegate(
  artifact: Artifact,
  delegates: Set<string>,
): boolean {
  return artifact.redactions.some(r => delegates.has(r.user.did));
}

// Group locations by the contributing node, preserving order.
export function locationsByNode(
  locations: Location[],
): { user: Author; urls: string[] }[] {
  const groups = new Map<string, { user: Author; urls: string[] }>();
  for (const { user, url } of locations) {
    const group = groups.get(user.did) ?? { user, urls: [] };
    group.urls.push(url);
    groups.set(user.did, group);
  }
  return [...groups.values()];
}

// Order a node-keyed list so delegates come first. The backend returns these
// sorted by DID (arbitrary to a reader); sort is stable, so the original
// order is preserved within each group.
export function delegatesFirst<T>(
  items: T[],
  did: (item: T) => string,
  delegates: Set<string>,
): T[] {
  return [...items].sort(
    (a, b) => Number(delegates.has(did(b))) - Number(delegates.has(did(a))),
  );
}

// Locations a browser can open, as opposed to ones fetched from a seeder
// over the radicle-artifact protocol. Delegate locations come first: they
// are the ones a reader can trust most.
export function webLocations(
  locations: Location[],
  delegates: Set<string>,
): Location[] {
  return delegatesFirst(
    locations.filter(l => /^https?:\/\//i.test(l.url)),
    l => l.user.did,
    delegates,
  );
}

// The trust signal worth carrying on the collapsed row. Only delegates count:
// anyone can make keys that attest their own artifact.
export function attestedBy(
  attestations: Author[],
  delegates: Set<string>,
): number {
  return attestations.filter(a => delegates.has(a.did)).length;
}

// Only metadata by the author or a delegate is shown, so only they get an
// editor.
export function canEditMetadata(
  artifact: Artifact,
  ownDid: string,
  delegates: Set<string>,
): boolean {
  return artifact.author.did === ownDid || delegates.has(ownDid);
}

// The author vouches by registering, so the COB ignores their attestation,
// and a redaction bars the same user from attesting.
export function canAttest(artifact: Artifact, ownDid: string): boolean {
  return (
    artifact.author.did !== ownDid &&
    !artifact.redacted &&
    !artifact.attestations.some(a => a.did === ownDid) &&
    !artifact.redactions.some(r => r.user.did === ownDid)
  );
}

// The COB stores values as free-form JSON. Text that parses as JSON is sent
// as that value, so numbers and booleans round-trip; anything else is sent
// as a plain string.
export function parseMetadataValue(input: string): unknown {
  try {
    return JSON.parse(input);
  } catch {
    return input;
  }
}

export function displayMetadataValue(value: unknown): string {
  return typeof value === "string" ? value : JSON.stringify(value);
}

// Commits whose SHA starts with, or whose summary contains, the query. A
// pasted commit outside the loaded history still shows as a choice.
export function matchCommits(
  commits: Commit[],
  query: string,
  resolved: Commit | undefined,
): Commit[] {
  const q = query.trim().toLowerCase();
  const matches = commits.filter(
    c => c.id.startsWith(q) || c.summary.toLowerCase().includes(q),
  );
  if (resolved && !matches.some(c => c.id === resolved.id)) {
    return [resolved];
  }
  return matches;
}

// Only canonical tags: a peer's tag of the same name could point a release
// at a commit the delegates never tagged.
export function canonicalTags(refs: RepoRefs): { name: string; tag: Tag }[] {
  return Object.entries(refs.canonical.tags)
    .map(([name, tag]) => ({ name, tag }))
    .sort((a, b) => b.tag.timestamp - a.tag.timestamp);
}
