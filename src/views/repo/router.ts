import type { Action as IssueAction } from "@bindings/cob/issue/Action";
import type { Action as PatchAction } from "@bindings/cob/patch/Action";
import type { Config } from "@bindings/config/Config";
import type { Issue } from "@bindings/cob/issue/Issue";
import type { Operation } from "@bindings/cob/Operation";
import type { PaginatedQuery } from "@bindings/cob/PaginatedQuery";
import type { Patch } from "@bindings/cob/patch/Patch";
import type { RepoInfo } from "@bindings/repo/RepoInfo";
import type { Review } from "@bindings/cob/patch/Review";
import type { Revision } from "@bindings/cob/patch/Revision";
import type { Thread } from "@bindings/cob/thread/Thread";

import { invoke } from "@app/lib/invoke";
import { unreachable } from "@app/lib/utils";

export type IssueStatus = "all" | Issue["state"]["status"];

export const DEFAULT_TAKE = 20;

export interface RepoIssueRoute {
  resource: "repo.issue";
  rid: string;
  issue: string;
  status: IssueStatus;
}

export interface RepoCreateIssueRoute {
  resource: "repo.createIssue";
  rid: string;
  status: IssueStatus;
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
  };
}

export interface LoadedRepoCreateIssueRoute {
  resource: "repo.createIssue";
  params: {
    repo: RepoInfo;
    config: Config;
    issues: Issue[];
    status: IssueStatus;
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
    config: Config;
    issues: Issue[];
    status: IssueStatus;
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
    review: Review | undefined;
    revisions: Revision[];
    activity: Operation<PatchAction>[];
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
    config: Config;
    patches: PaginatedQuery<Patch[]>;
    status: PatchStatus | undefined;
  };
}

export type RepoRoute =
  | RepoCreateIssueRoute
  | RepoIssueRoute
  | RepoIssuesRoute
  | RepoPatchRoute
  | RepoPatchesRoute;
export type LoadedRepoRoute =
  | LoadedRepoCreateIssueRoute
  | LoadedRepoIssueRoute
  | LoadedRepoIssuesRoute
  | LoadedRepoPatchRoute
  | LoadedRepoPatchesRoute;

export async function loadPatch(
  route: RepoPatchRoute,
): Promise<LoadedRepoPatchRoute> {
  const [config, repo, patches, patch, revisions, activity] = await Promise.all(
    [
      invoke<Config>("config"),
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
    ],
  );

  const review = revisions
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
    },
  };
}

export async function loadPatches(
  route: RepoPatchesRoute,
): Promise<LoadedRepoPatchesRoute> {
  const [config, repo, patches] = await Promise.all([
    invoke<Config>("config"),
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
    params: { repo, config, patches, status: route.status },
  };
}

export async function loadCreateIssue(
  route: RepoCreateIssueRoute,
): Promise<LoadedRepoCreateIssueRoute> {
  const [config, repo, issues] = await Promise.all([
    invoke<Config>("config"),
    invoke<RepoInfo>("repo_by_id", {
      rid: route.rid,
    }),
    invoke<Issue[]>("list_issues", {
      rid: route.rid,
      status: route.status,
    }),
  ]);

  return {
    resource: "repo.createIssue",
    params: { repo, config, issues, status: route.status },
  };
}

export async function loadIssue(
  route: RepoIssueRoute,
): Promise<LoadedRepoIssueRoute> {
  const [config, repo, issue, activity, issues, threads] = await Promise.all([
    invoke<Config>("config"),
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
      repo,
      config,
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
  const [config, repo, issues] = await Promise.all([
    invoke<Config>("config"),
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
    params: { repo, config, issues, status: route.status },
  };
}

export function repoRouteToPath(route: RepoRoute): string {
  const pathSegments = ["/repos", route.rid];
  const searchParams = new URLSearchParams();

  if (route.resource === "repo.issue") {
    let url = [...pathSegments, "issues", route.issue].join("/");
    searchParams.set("status", route.status);
    url += `?${searchParams}`;
    return url;
  } else if (route.resource === "repo.createIssue") {
    let url = [...pathSegments, "issues", "create"].join("/");
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
    if (resource === "issues") {
      const idOrAction = segments.shift();
      if (idOrAction) {
        const status = (searchParams.get("status") ?? "all") as IssueStatus;
        if (idOrAction === "create") {
          return { resource: "repo.createIssue", rid, status };
        } else {
          return {
            resource: "repo.issue",
            rid,
            issue: idOrAction,
            status,
          };
        }
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
