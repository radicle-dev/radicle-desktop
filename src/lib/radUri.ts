import bs58 from "bs58";

export const issueType = "xyz.radicle.issue";
export const patchType = "xyz.radicle.patch";
export const releaseType = "dev.radicle.artifact";

const explorerCobPaths: Record<string, string> = {
  [issueType]: "issues",
  [patchType]: "patches",
  [releaseType]: "releases",
};

export interface RadAuthority {
  node: string;
  address?: string;
}

export type RadResource =
  | { type: "commit"; ref: string }
  | { type: "tag"; ref: string }
  | { type: "tree"; oid: string }
  | { type: "blob"; oid: string }
  | { type: "cob"; typeName: string; oid?: string };

export interface QueryParam {
  param: string;
  value?: string;
}

export interface RadUri {
  authority?: RadAuthority;
  repo: string;
  namespace?: string;
  resource?: RadResource;
  query?: QueryParam[];
  fragment?: string;
}

export type RadReference =
  { type: "uri"; uri: RadUri } | { type: "did"; node: string };

const base58 = "[1-9A-HJ-NP-Za-km-z]";
const repoIdPattern = new RegExp(`^z${base58}{27,28}$`);
const nodeIdPattern = new RegExp(`^z6Mk${base58}{44}$`);
const oidPattern = /^[0-9a-fA-F]{40}$/;
const gitRefPattern = /^[A-Za-z0-9._~-]+(?:\/[A-Za-z0-9._~-]+)*$/;
const cobTypePattern =
  /^[A-Za-z0-9]+(?:-[A-Za-z0-9]+)*(?:\.[A-Za-z0-9]+(?:-[A-Za-z0-9]+)*)+$/;
const hostPattern =
  /^(?:\[[0-9A-Fa-f:.vV]+\]|(?:[A-Za-z0-9._~!$&'()*+,;=-]|%[0-9A-Fa-f]{2})*)$/;
const portPattern = /^[0-9]*$/;
const queryOrFragmentPattern =
  /^(?:[A-Za-z0-9._~!$&'()*+,;=:@/?-]|%[0-9A-Fa-f]{2})*$/;

function decodesTo(id: string, check: (bytes: Uint8Array) => boolean) {
  try {
    return check(bs58.decode(id.slice(1)));
  } catch {
    return false;
  }
}

export function isRepoId(id: string): boolean {
  return (
    repoIdPattern.test(id) && decodesTo(id, bytes => bytes.byteLength === 20)
  );
}

export function isNodeId(id: string): boolean {
  return (
    nodeIdPattern.test(id) &&
    decodesTo(
      id,
      bytes =>
        bytes.byteLength === 34 && bytes[0] === 0xed && bytes[1] === 0x01,
    )
  );
}

export function isOid(oid: string): boolean {
  return oidPattern.test(oid);
}

function parseAuthority(input: string): RadAuthority | undefined {
  const at = input.indexOf("@");
  const node = at === -1 ? input : input.slice(0, at);
  if (!isNodeId(node)) return undefined;
  if (at === -1) return { node };

  const address = input.slice(at + 1);
  const colon = address.lastIndexOf(":");
  if (colon === -1) return undefined;
  if (
    !hostPattern.test(address.slice(0, colon)) ||
    !portPattern.test(address.slice(colon + 1))
  ) {
    return undefined;
  }

  return { node, address };
}

function parseResource(segments: string[]): RadResource | undefined | null {
  if (segments.length === 0) return undefined;

  const [kind, ...rest] = segments;
  switch (kind.toLowerCase()) {
    case "commit":
    case "tag": {
      const ref = rest.join("/");
      if (!gitRefPattern.test(ref)) return null;
      return {
        type: kind.toLowerCase() as "commit" | "tag",
        ref: isOid(ref) ? ref.toLowerCase() : ref,
      };
    }
    case "tree":
    case "blob":
      if (rest.length !== 1 || !isOid(rest[0])) return null;
      return {
        type: kind.toLowerCase() as "tree" | "blob",
        oid: rest[0].toLowerCase(),
      };
    case "cob": {
      const [typeName, oid, ...extra] = rest;
      if (extra.length > 0 || !typeName || !cobTypePattern.test(typeName)) {
        return null;
      }
      if (oid === undefined) return { type: "cob", typeName };
      if (!isOid(oid)) return null;
      return { type: "cob", typeName, oid: oid.toLowerCase() };
    }
    default:
      return null;
  }
}

function parseQuery(input: string): QueryParam[] {
  return input.split("&").map(pair => {
    const equals = pair.indexOf("=");
    return equals === -1
      ? { param: pair }
      : { param: pair.slice(0, equals), value: pair.slice(equals + 1) };
  });
}

export function parseRadUri(input: string): RadUri | undefined {
  if (input.slice(0, 4).toLowerCase() !== "rad:") return undefined;
  let rest = input.slice(4);

  let fragment: string | undefined;
  const hash = rest.indexOf("#");
  if (hash !== -1) {
    fragment = rest.slice(hash + 1);
    rest = rest.slice(0, hash);
    if (!queryOrFragmentPattern.test(fragment)) return undefined;
  }

  let query: QueryParam[] | undefined;
  const question = rest.indexOf("?");
  if (question !== -1) {
    const raw = rest.slice(question + 1);
    rest = rest.slice(0, question);
    if (!queryOrFragmentPattern.test(raw)) return undefined;
    query = parseQuery(raw);
  }

  let authority: RadAuthority | undefined;
  let legacy = false;
  if (rest.startsWith("//")) {
    rest = rest.slice(2);
    if (rest.startsWith("/")) {
      rest = rest.slice(1);
    } else {
      const slash = rest.indexOf("/");
      const head = slash === -1 ? rest : rest.slice(0, slash);
      if (isRepoId(head)) {
        legacy = true;
      } else {
        authority = parseAuthority(head);
        if (!authority || slash === -1) return undefined;
        rest = rest.slice(slash + 1);
      }
    }
  }

  const segments = rest.split("/");
  const repo = segments.shift() ?? "";
  if (!isRepoId(repo)) return undefined;

  let namespace: string | undefined;
  if (segments.length > 0 && nodeIdPattern.test(segments[0])) {
    namespace = segments.shift();
    if (!namespace || !isNodeId(namespace)) return undefined;
  }

  const resource = parseResource(segments);
  if (resource === null) return undefined;
  if (legacy && (resource || query || fragment !== undefined)) {
    return undefined;
  }

  return { authority, repo, namespace, resource, query, fragment };
}

function formatResource(resource: RadResource): string {
  switch (resource.type) {
    case "commit":
    case "tag":
      return `/${resource.type}/${resource.ref}`;
    case "tree":
    case "blob":
      return `/${resource.type}/${resource.oid}`;
    case "cob":
      return resource.oid === undefined
        ? `/cob/${resource.typeName}`
        : `/cob/${resource.typeName}/${resource.oid}`;
  }
}

export function formatRadUri(
  uri: RadUri,
  options: { forceAuthority?: boolean } = {},
): string {
  let result = "rad:";
  if (uri.authority) {
    const { node, address } = uri.authority;
    result += `//${node}${address === undefined ? "" : `@${address}`}/`;
  } else if (options.forceAuthority) {
    result += "///";
  }
  result += uri.repo;
  if (uri.namespace) result += `/${uri.namespace}`;
  if (uri.resource) result += formatResource(uri.resource);
  if (uri.query && uri.query.length > 0) {
    result += `?${uri.query
      .map(({ param, value }) =>
        value === undefined ? param : `${param}=${value}`,
      )
      .join("&")}`;
  }
  if (uri.fragment !== undefined) result += `#${uri.fragment}`;

  return result;
}

export function parseDid(input: string): string | undefined {
  if (!input.startsWith("did:key:")) return undefined;
  const node = input.slice(8);

  return isNodeId(node) ? node : undefined;
}

export function formatDid(node: string): string {
  return `did:key:${node}`;
}

export function parseReference(input: string): RadReference | undefined {
  const node = parseDid(input);
  if (node) return { type: "did", node };
  const uri = parseRadUri(input);

  return uri ? { type: "uri", uri } : undefined;
}

export function formatReference(reference: RadReference): string {
  return reference.type === "did"
    ? formatDid(reference.node)
    : formatRadUri(reference.uri);
}

function queryPath(uri: RadUri): string | undefined {
  return uri.query?.find(
    ({ param, value }) =>
      (param === "path" || param === "tree" || param === "blob") &&
      value !== undefined &&
      value !== "",
  )?.value;
}

function explorerRevision(ref: string): string | undefined {
  if (!ref.startsWith("refs/")) return ref;
  for (const prefix of ["refs/heads/", "refs/tags/"]) {
    if (ref.startsWith(prefix)) return ref.slice(prefix.length);
  }

  return undefined;
}

function explorerResourcePath(uri: RadUri): string | undefined {
  const { resource } = uri;
  const remote = uri.namespace ? `/remotes/${uri.namespace}` : "";
  if (!resource) return remote;

  switch (resource.type) {
    case "commit":
    case "tag": {
      const revision = explorerRevision(resource.ref);
      if (revision === undefined) return undefined;
      const path = queryPath(uri);
      if (path !== undefined) return `${remote}/tree/${revision}/${path}`;
      if (resource.type === "commit" && isOid(revision)) {
        return `/commits/${revision}`;
      }
      return `${remote}/tree/${revision}`;
    }
    case "tree":
    case "blob":
      return undefined;
    case "cob": {
      const kind = explorerCobPaths[resource.typeName];
      if (!kind) return undefined;
      return resource.oid === undefined
        ? `/${kind}`
        : `/${kind}/${resource.oid}`;
    }
  }
}

export function explorerUrl(
  reference: RadReference,
  base: string,
  host: string,
): string | undefined {
  const node = `${base.replace(/\/+$/, "")}/nodes/${host}`;
  if (reference.type === "did") {
    return `${node}/users/${formatDid(reference.node)}`;
  }

  const path = explorerResourcePath(reference.uri);
  if (path === undefined) return undefined;

  return `${node}/rad:${reference.uri.repo}${path}`;
}

function decodeSegment(segment: string): string {
  try {
    return decodeURIComponent(segment);
  } catch {
    return segment;
  }
}

export function parseExplorerUrl(href: string): RadReference | undefined {
  let url: URL;
  try {
    url = new URL(href);
  } catch {
    return undefined;
  }
  if (url.protocol !== "https:" && url.protocol !== "http:") return undefined;

  const segments = url.pathname
    .split("/")
    .filter(segment => segment !== "")
    .map(decodeSegment);

  const users = segments.indexOf("users");
  if (users !== -1 && users === segments.length - 2) {
    const node = parseDid(segments[users + 1]);
    return node ? { type: "did", node } : undefined;
  }

  const index = segments.findIndex(segment => segment.startsWith("rad:"));
  if (index === -1) return undefined;

  const repo = segments[index].slice(4);
  if (!isRepoId(repo)) return undefined;

  let tail = segments.slice(index + 1);
  let namespace: string | undefined;
  if (tail[0] === "remotes") {
    namespace = tail[1];
    if (namespace === undefined || !isNodeId(namespace)) return undefined;
    tail = tail.slice(2);
  }

  const [kind, ...rest] = tail;
  if (kind === undefined || (kind === "tree" && rest.length === 0)) {
    return { type: "uri", uri: { repo, namespace } };
  }

  if (kind === "tree") {
    const [revision, ...path] = rest;
    if (isOid(revision)) {
      return {
        type: "uri",
        uri: {
          repo,
          namespace,
          resource: { type: "commit", ref: revision.toLowerCase() },
          query:
            path.length > 0
              ? [{ param: "path", value: path.join("/") }]
              : undefined,
        },
      };
    }
    if (path.length > 0 || !gitRefPattern.test(revision)) return undefined;
    return {
      type: "uri",
      uri: { repo, namespace, resource: { type: "commit", ref: revision } },
    };
  }

  const [raw, ...extra] = rest;
  if ((extra.length > 0 && kind !== "patches") || !raw || !isOid(raw)) {
    return undefined;
  }
  const oid = raw.toLowerCase();

  if (kind === "commits") {
    return {
      type: "uri",
      uri: { repo, resource: { type: "commit", ref: oid } },
    };
  }

  const typeName = Object.keys(explorerCobPaths).find(
    key => explorerCobPaths[key] === kind,
  );
  if (!typeName) return undefined;

  return {
    type: "uri",
    uri: { repo, resource: { type: "cob", typeName, oid } },
  };
}
