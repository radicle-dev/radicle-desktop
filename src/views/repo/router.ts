import type { Config } from "@bindings/Config";
import type { Issue } from "@bindings/Issue";
import type { Patch } from "@bindings/Patch";
import type { RepoInfo } from "@bindings/RepoInfo";

import { invoke } from "@tauri-apps/api/core";
import { unreachable } from "@app/lib/utils";

export type IssueStatus = "all" | Issue["state"]["status"];

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

export type PatchStatus = "all" | Patch["state"]["status"];

export interface RepoPatchesRoute {
  resource: "repo.patches";
  rid: string;
  status: PatchStatus;
}

export interface LoadedRepoPatchesRoute {
  resource: "repo.patches";
  params: {
    repo: RepoInfo;
    config: Config;
    patches: Patch[];
    status: PatchStatus;
  };
}

export type RepoRoute = RepoIssuesRoute | RepoPatchesRoute;
export type LoadedRepoRoute = LoadedRepoIssuesRoute | LoadedRepoPatchesRoute;

export async function loadPatches(
  route: RepoPatchesRoute,
): Promise<LoadedRepoPatchesRoute> {
  const repo: RepoInfo = await invoke("repo_by_id", {
    rid: route.rid,
  });
  const config: Config = await invoke("config");
  const patches: Patch[] = await invoke("list_patches", {
    rid: route.rid,
    status: route.status,
  });

  return {
    resource: "repo.patches",
    params: { repo, config, patches, status: route.status },
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

  if (route.resource === "repo.issues") {
    let url = [...pathSegments, "issues"].join("/");
    const searchParams = new URLSearchParams();
    if (route.status) {
      searchParams.set("status", route.status);
      url += `?${searchParams}`;
    }
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
      const status = searchParams.get("status");
      if (status === "open" || status === "closed") {
        return { resource: "repo.issues", rid, status };
      } else {
        return { resource: "repo.issues", rid, status: "all" };
      }
    } else if (resource === "patches") {
      const status = searchParams.get("status");
      if (
        status === "draft" ||
        status === "open" ||
        status === "archived" ||
        status === "merged"
      ) {
        return { resource: "repo.patches", rid, status };
      } else {
        return { resource: "repo.patches", rid, status: "all" };
      }
    } else {
      return null;
    }
  } else {
    return null;
  }
}
