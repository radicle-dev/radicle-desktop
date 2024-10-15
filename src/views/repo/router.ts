import type { Config } from "@bindings/config/Config";
import type { PaginatedQuery } from "@bindings/cob/PaginatedQuery";
import type { Issue } from "@bindings/cob/issue/Issue";
import type { Patch } from "@bindings/cob/patch/Patch";
import type { RepoInfo } from "@bindings/repo/RepoInfo";
import type { Revision } from "@bindings/cob/patch/Revision";

import { invoke } from "@tauri-apps/api/core";
import { unreachable } from "@app/lib/utils";

export type IssueStatus = "all" | Issue["state"]["status"];

export interface RepoIssueRoute {
  resource: "repo.issue";
  rid: string;
  issue: string;
}

export interface RepoCreateIssueRoute {
  resource: "repo.createIssue";
  rid: string;
}

export interface LoadedRepoIssueRoute {
  resource: "repo.issue";
  params: {
    repo: RepoInfo;
    config: Config;
    issue: Issue;
    issues: Issue[];
  };
}

export interface LoadedRepoCreateIssueRoute {
  resource: "repo.createIssue";
  params: {
    repo: RepoInfo;
    config: Config;
    issues: Issue[];
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
}

export interface LoadedRepoPatchRoute {
  resource: "repo.patch";
  params: {
    repo: RepoInfo;
    config: Config;
    patch: Patch;
    patches: Patch[];
    revisions: Revision[];
  };
}

export interface RepoPatchesRoute {
  resource: "repo.patches";
  rid: string;
  status?: PatchStatus;
}

export interface LoadedRepoPatchesRoute {
  resource: "repo.patches";
  params: {
    repo: RepoInfo;
    config: Config;
    patches: PaginatedQuery<Patch[]>;
    status?: PatchStatus;
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
  const repo: RepoInfo = await invoke("repo_by_id", {
    rid: route.rid,
  });
  const config: Config = await invoke("config");
  const patches: PaginatedQuery<Patch[]> = await invoke("list_patches", {
    rid: route.rid,
  });
  const patch: Patch = await invoke("patch_by_id", {
    rid: route.rid,
    id: route.patch,
  });
  const revisions: Revision[] = await invoke("revisions_by_patch", {
    rid: route.rid,
    id: route.patch,
  });

  return {
    resource: "repo.patch",
    params: { repo, config, patch, patches: patches.content, revisions },
  };
}

export async function loadPatches(
  route: RepoPatchesRoute,
): Promise<LoadedRepoPatchesRoute> {
  const repo: RepoInfo = await invoke("repo_by_id", {
    rid: route.rid,
  });
  const config: Config = await invoke("config");
  const patches: PaginatedQuery<Patch[]> = await invoke("list_patches", {
    rid: route.rid,
    status: route.status,
  });

  return {
    resource: "repo.patches",
    params: { repo, config, patches, status: route.status },
  };
}

export async function loadCreateIssue(
  route: RepoCreateIssueRoute,
): Promise<LoadedRepoCreateIssueRoute> {
  const repo: RepoInfo = await invoke("repo_by_id", {
    rid: route.rid,
  });
  const config: Config = await invoke("config");
  const issues: Issue[] = await invoke("list_issues", {
    rid: route.rid,
    status: "all",
  });

  return {
    resource: "repo.createIssue",
    params: { repo, config, issues },
  };
}

export async function loadIssue(
  route: RepoIssueRoute,
): Promise<LoadedRepoIssueRoute> {
  const repo: RepoInfo = await invoke("repo_by_id", {
    rid: route.rid,
  });
  const config: Config = await invoke("config");
  const issues: Issue[] = await invoke("list_issues", {
    rid: route.rid,
    status: "all",
  });
  const issue: Issue = await invoke("issue_by_id", {
    rid: route.rid,
    id: route.issue,
  });

  return {
    resource: "repo.issue",
    params: { repo, config, issue, issues },
  };
}

export async function loadIssues(
  route: RepoIssuesRoute,
): Promise<LoadedRepoIssuesRoute> {
  const repo: RepoInfo = await invoke("repo_by_id", {
    rid: route.rid,
  });
  const config: Config = await invoke("config");
  const issues: Issue[] = await invoke("list_issues", {
    rid: route.rid,
    status: route.status,
  });

  return {
    resource: "repo.issues",
    params: { repo, config, issues, status: route.status },
  };
}

export function repoRouteToPath(route: RepoRoute): string {
  const pathSegments = ["/repos", route.rid];

  if (route.resource === "repo.issue") {
    const url = [...pathSegments, "issues", route.issue].join("/");
    return url;
  } else if (route.resource === "repo.createIssue") {
    const url = [...pathSegments, "issues", "create"].join("/");
    return url;
  } else if (route.resource === "repo.issues") {
    let url = [...pathSegments, "issues"].join("/");
    const searchParams = new URLSearchParams();
    if (route.status) {
      searchParams.set("status", route.status);
      url += `?${searchParams}`;
    }
    return url;
  } else if (route.resource === "repo.patch") {
    const url = [...pathSegments, "patches", route.patch].join("/");
    return url;
  } else if (route.resource === "repo.patches") {
    let url = [...pathSegments, "patches"].join("/");
    const searchParams = new URLSearchParams();
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
        if (idOrAction === "create") {
          return { resource: "repo.createIssue", rid };
        } else {
          return {
            resource: "repo.issue",
            rid,
            issue: idOrAction,
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
      if (id) {
        return {
          resource: "repo.patch",
          rid,
          patch: id,
        };
      } else {
        const status = searchParams.get("status");
        if (
          status === "draft" ||
          status === "open" ||
          status === "archived" ||
          status === "merged"
        ) {
          return { resource: "repo.patches", rid, status };
        } else {
          return { resource: "repo.patches", rid };
        }
      }
    } else {
      return null;
    }
  } else {
    return null;
  }
}
