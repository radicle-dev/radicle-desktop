import type { Action as IssueAction } from "@bindings/cob/issue/Action";
import type { Issue } from "@bindings/cob/issue/Issue";
import type { Operation } from "@bindings/cob/Operation";
import type { PaginatedQuery } from "@bindings/cob/PaginatedQuery";
import type { Action as PatchAction } from "@bindings/cob/patch/Action";
import type { Patch } from "@bindings/cob/patch/Patch";
import type { Review } from "@bindings/cob/patch/Review";
import type { Revision } from "@bindings/cob/patch/Revision";
import type { Thread } from "@bindings/cob/thread/Thread";
import type { Config } from "@bindings/config/Config";
import type { Diff } from "@bindings/diff/Diff";
import type { Commit } from "@bindings/repo/Commit";
import type { Readme } from "@bindings/repo/Readme";
import type { RepoInfo } from "@bindings/repo/RepoInfo";
import type { RepoRefs } from "@bindings/repo/RepoRefs";
import type { Tree } from "@bindings/source/Tree";

import type { DraftReview } from "@app/lib/draftReviewStorage";
import { draftReviewStorage } from "@app/lib/draftReviewStorage";
import { cachedGetCommitDiff, invoke } from "@app/lib/invoke";
import type { SidebarData } from "@app/lib/router/definitions";
import { loadSidebarData } from "@app/lib/router/definitions";
import { didFromPublicKey, unreachable } from "@app/lib/utils";

export type IssueStatus = "all" | Issue["state"]["status"];

export const DEFAULT_TAKE = 20;
export const COMMITS_PAGE_SIZE = 300;

export interface RepoHomeRoute {
  resource: "repo.home";
  rid: string;
  peer?: string;
  revision?: string;
}

export interface RepoCommitsRoute {
  resource: "repo.commits";
  rid: string;
}

export interface RepoCommitRoute {
  resource: "repo.commit";
  rid: string;
  commit: string;
}

export interface RepoIssueRoute {
  resource: "repo.issue";
  rid: string;
  issue: string;
  status: IssueStatus;
}

export interface LoadedRepoHomeRoute {
  resource: "repo.home";
  params: {
    repo: RepoInfo;
    refs: RepoRefs;
    peer?: string;
    revision?: string;
    oid: string;
    tree: Tree;
    readme: Readme | null;
    sidebarData: SidebarData;
  };
}

export interface LoadedRepoCommitsRoute {
  resource: "repo.commits";
  params: {
    repo: RepoInfo;
    commits: PaginatedQuery<Commit[]>;
    sidebarData: SidebarData;
  };
}

export interface LoadedRepoCommitRoute {
  resource: "repo.commit";
  params: {
    repo: RepoInfo;
    commit: Commit;
    diff: Diff;
    sidebarData: SidebarData;
  };
}

export interface LoadedRepoIssueRoute {
  resource: "repo.issue";
  params: {
    repo: RepoInfo;
    config: Config;
    issue: Issue;
    issues: Issue[];
    status: IssueStatus;
    activity: Operation<IssueAction>[];
    threads: Thread[];
    sidebarData: SidebarData;
  };
}

export interface RepoIssuesRoute {
  resource: "repo.issues";
  rid: string;
  status: IssueStatus;
}

export interface LoadedRepoIssuesRoute {
  resource: "repo.issues";
  params: {
    repo: RepoInfo;
    issues: Issue[];
    status: IssueStatus;
    sidebarData: SidebarData;
  };
}

export type PatchStatus = Patch["state"]["status"];

export interface RepoPatchRoute {
  resource: "repo.patch";
  rid: string;
  patch: string;
  status: PatchStatus | undefined;
  reviewId: string | undefined;
}

export interface LoadedRepoPatchRoute {
  resource: "repo.patch";
  params: {
    repo: RepoInfo;
    config: Config;
    patch: Patch;
    patches: PaginatedQuery<Patch[]>;
    status: PatchStatus | undefined;
    review: Review | DraftReview | undefined;
    revisions: Revision[];
    activity: Operation<PatchAction>[];
    sidebarData: SidebarData;
  };
}

export interface RepoPatchesRoute {
  resource: "repo.patches";
  rid: string;
  status: PatchStatus | undefined;
}

export interface LoadedRepoPatchesRoute {
  resource: "repo.patches";
  params: {
    repo: RepoInfo;
    patches: PaginatedQuery<Patch[]>;
    status: PatchStatus | undefined;
    sidebarData: SidebarData;
  };
}

export type RepoRoute =
  | RepoHomeRoute
  | RepoCommitsRoute
  | RepoCommitRoute
  | RepoIssueRoute
  | RepoIssuesRoute
  | RepoPatchRoute
  | RepoPatchesRoute;
export type LoadedRepoRoute =
  | LoadedRepoHomeRoute
  | LoadedRepoCommitsRoute
  | LoadedRepoCommitRoute
  | LoadedRepoIssueRoute
  | LoadedRepoIssuesRoute
  | LoadedRepoPatchRoute
  | LoadedRepoPatchesRoute;

export async function loadPatch(
  route: RepoPatchRoute,
): Promise<LoadedRepoPatchRoute> {
  const [sidebarData, repo, patches, patch, revisions, activity] =
    await Promise.all([
      loadSidebarData(),
      invoke<RepoInfo>("repo_by_id", {
        rid: route.rid,
      }),
      invoke<PaginatedQuery<Patch[]>>("list_patches", {
        rid: route.rid,
        status: route.status,
        take: DEFAULT_TAKE,
      }),
      invoke<Patch>("patch_by_id", {
        rid: route.rid,
        id: route.patch,
      }),
      invoke<Revision[]>("revisions_by_patch", {
        rid: route.rid,
        id: route.patch,
      }),
      invoke<Operation<PatchAction>[]>("activity_by_patch", {
        rid: route.rid,
        id: route.patch,
      }),
    ]);

  const config = sidebarData.config;

  const draftReview =
    route.reviewId !== undefined &&
    draftReviewStorage.get(route.reviewId, {
      did: didFromPublicKey(config.publicKey),
      alias: config.alias,
    });

  const review =
    draftReview ||
    revisions
      .flatMap(r => r.reviews || [])
      .find(review => review.id === route.reviewId);

  return {
    resource: "repo.patch",
    params: {
      repo,
      config,
      patch,
      patches,
      revisions,
      status: route.status,
      review,
      activity,
      sidebarData,
    },
  };
}

export async function loadPatches(
  route: RepoPatchesRoute,
): Promise<LoadedRepoPatchesRoute> {
  const [sidebarData, repo, patches] = await Promise.all([
    loadSidebarData(),
    invoke<RepoInfo>("repo_by_id", {
      rid: route.rid,
    }),
    invoke<PaginatedQuery<Patch[]>>("list_patches", {
      rid: route.rid,
      status: route.status,
      take: DEFAULT_TAKE,
    }),
  ]);

  return {
    resource: "repo.patches",
    params: { sidebarData, repo, patches, status: route.status },
  };
}

export async function loadRepoHome(
  route: RepoHomeRoute,
): Promise<LoadedRepoHomeRoute> {
  const [sidebarData, repo, refs] = await Promise.all([
    loadSidebarData(),
    invoke<RepoInfo>("repo_by_id", {
      rid: route.rid,
    }),
    invoke<RepoRefs>("list_repo_refs", {
      rid: route.rid,
    }),
  ]);

  const oid = resolveRevision({
    repo,
    refs,
    peer: route.peer,
    revision: route.revision,
  });

  const [readme, tree] = await Promise.all([
    invoke<Readme | null>("repo_readme", {
      rid: route.rid,
      sha: oid,
    }),
    invoke<Tree>("repo_tree", {
      rid: route.rid,
      path: "",
      sha: oid,
    }),
  ]);

  return {
    resource: "repo.home",
    params: {
      sidebarData,
      repo,
      refs,
      peer: route.peer,
      revision: route.revision,
      oid,
      readme,
      tree,
    },
  };
}

export function resolveRevision({
  repo,
  refs,
  peer,
  revision,
}: {
  repo: RepoInfo;
  refs: RepoRefs;
  peer?: string;
  revision?: string;
}): string {
  const project = repo.payloads["xyz.radicle.project"];
  const defaultBranch = project?.data.defaultBranch;

  if (peer !== undefined) {
    const remote = refs.remotes.find(r => r.id === peer);
    if (!remote) {
      throw new Error(`Peer ${peer} not found in repo ${repo.rid}`);
    }
    const refName = revision ?? defaultBranch;
    if (refName !== undefined) {
      if (refName in remote.branches) {
        return remote.branches[refName];
      }
      if (refName in remote.tags) {
        return remote.tags[refName].oid;
      }
    }
    if (revision !== undefined) {
      return revision;
    }
    throw new Error(`Cannot resolve revision for peer ${peer}`);
  }

  if (revision !== undefined) {
    if (revision in refs.canonical.branches) {
      return refs.canonical.branches[revision];
    }
    if (revision in refs.canonical.tags) {
      return refs.canonical.tags[revision].oid;
    }
    if (revision === defaultBranch && project) {
      return project.meta.head;
    }
    return revision;
  }

  if (project) {
    return project.meta.head;
  }
  throw new Error(`Cannot resolve revision for repo ${repo.rid}`);
}

export async function loadRepoCommits(
  route: RepoCommitsRoute,
): Promise<LoadedRepoCommitsRoute> {
  const [sidebarData, repo, commits] = await Promise.all([
    loadSidebarData(),
    invoke<RepoInfo>("repo_by_id", {
      rid: route.rid,
    }),
    invoke<PaginatedQuery<Commit[]>>("list_repo_commits", {
      rid: route.rid,
      skip: 0,
      take: COMMITS_PAGE_SIZE,
    }),
  ]);

  return {
    resource: "repo.commits",
    params: { sidebarData, repo, commits },
  };
}

export async function loadRepoCommit(
  route: RepoCommitRoute,
): Promise<LoadedRepoCommitRoute> {
  const [sidebarData, repo, commit, diff] = await Promise.all([
    loadSidebarData(),
    invoke<RepoInfo>("repo_by_id", {
      rid: route.rid,
    }),
    invoke<Commit>("repo_commit", {
      rid: route.rid,
      sha: route.commit,
    }),
    cachedGetCommitDiff(route.rid, route.commit, 3, true),
  ]);

  return {
    resource: "repo.commit",
    params: { sidebarData, repo, commit, diff },
  };
}

export async function loadIssue(
  route: RepoIssueRoute,
): Promise<LoadedRepoIssueRoute> {
  const [sidebarData, repo, issue, activity, issues, threads] =
    await Promise.all([
      loadSidebarData(),
      invoke<RepoInfo>("repo_by_id", {
        rid: route.rid,
      }),
      invoke<Issue>("issue_by_id", {
        rid: route.rid,
        id: route.issue,
      }),
      invoke<Operation<IssueAction>[]>("activity_by_issue", {
        rid: route.rid,
        id: route.issue,
      }),
      invoke<Issue[]>("list_issues", {
        rid: route.rid,
        status: route.status,
      }),
      invoke<Thread[]>("comment_threads_by_issue_id", {
        rid: route.rid,
        id: route.issue,
      }),
    ]);

  return {
    resource: "repo.issue",
    params: {
      sidebarData,
      repo,
      config: sidebarData.config,
      issue,
      activity,
      issues,
      threads,
      status: route.status,
    },
  };
}

export async function loadIssues(
  route: RepoIssuesRoute,
): Promise<LoadedRepoIssuesRoute> {
  const [sidebarData, repo, issues] = await Promise.all([
    loadSidebarData(),
    invoke<RepoInfo>("repo_by_id", {
      rid: route.rid,
    }),
    invoke<Issue[]>("list_issues", {
      rid: route.rid,
      status: route.status,
    }),
  ]);

  return {
    resource: "repo.issues",
    params: { sidebarData, repo, issues, status: route.status },
  };
}

export function repoRouteToPath(route: RepoRoute): string {
  const pathSegments = ["/repos", route.rid];
  const searchParams = new URLSearchParams();

  if (route.resource === "repo.home") {
    const segments = [...pathSegments, "home"];
    if (route.peer !== undefined) {
      segments.push("remotes", route.peer);
    }
    if (route.revision !== undefined) {
      segments.push(route.revision);
    }
    return segments.join("/");
  } else if (route.resource === "repo.commits") {
    return [...pathSegments, "commits"].join("/");
  } else if (route.resource === "repo.commit") {
    return [...pathSegments, "commits", route.commit].join("/");
  } else if (route.resource === "repo.issue") {
    let url = [...pathSegments, "issues", route.issue].join("/");
    searchParams.set("status", route.status);
    url += `?${searchParams}`;
    return url;
  } else if (route.resource === "repo.issues") {
    let url = [...pathSegments, "issues"].join("/");
    searchParams.set("status", route.status);
    url += `?${searchParams}`;
    return url;
  } else if (route.resource === "repo.patch") {
    let url = [...pathSegments, "patches", route.patch].join("/");
    if (route.status) {
      searchParams.set("status", route.status);
    }
    if (route.reviewId) {
      searchParams.set("review", route.reviewId);
    }
    if (searchParams.size > 0) {
      url += `?${searchParams}`;
    }
    return url;
  } else if (route.resource === "repo.patches") {
    let url = [...pathSegments, "patches"].join("/");
    if (route.status) {
      searchParams.set("status", route.status);
      url += `?${searchParams}`;
    }
    return url;
  } else {
    return unreachable(route);
  }
}

export function repoUrlToRoute(
  segments: string[],
  searchParams: URLSearchParams,
): RepoRoute | null {
  const rid = segments.shift();
  const resource = segments.shift();

  if (rid) {
    if (resource === "home") {
      let peer: string | undefined;
      if (segments[0] === "remotes") {
        segments.shift();
        peer = segments.shift();
      }
      const revision = segments.length > 0 ? segments.join("/") : undefined;
      return { resource: "repo.home", rid, peer, revision };
    } else if (resource === "commits") {
      const sha = segments.shift();

      if (sha) {
        return { resource: "repo.commit", rid, commit: sha };
      }

      return { resource: "repo.commits", rid };
    } else if (resource === "issues") {
      const idOrAction = segments.shift();
      if (idOrAction) {
        const status = (searchParams.get("status") ?? "all") as IssueStatus;
        if (idOrAction !== "create") {
          return {
            resource: "repo.issue",
            rid,
            issue: idOrAction,
            status,
          };
        }
        return null;
      } else {
        const status = searchParams.get("status");
        if (status === "open" || status === "closed") {
          return { resource: "repo.issues", rid, status };
        } else {
          return { resource: "repo.issues", rid, status: "all" };
        }
      }
    } else if (resource === "patches") {
      const id = segments.shift();
      const status = (searchParams.get("status") ?? undefined) as
        | PatchStatus
        | undefined;
      const reviewId = searchParams.get("review") ?? undefined;
      if (id) {
        return {
          resource: "repo.patch",
          rid,
          patch: id,
          status,
          reviewId,
        };
      } else {
        return { resource: "repo.patches", rid, status };
      }
    } else {
      return null;
    }
  } else {
    return null;
  }
}
