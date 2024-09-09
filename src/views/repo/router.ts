import type { RepoInfo } from "@bindings/RepoInfo";
import type { Patch } from "@bindings/Patch";
import type { Issue } from "@bindings/Issue";

import { invoke } from "@tauri-apps/api/core";
import { unreachable } from "@app/lib/utils";

export interface RepoIssuesRoute {
  resource: "repo.issues";
  rid: string;
  status?: "open" | "closed";
}

export interface LoadedRepoIssuesRoute {
  resource: "repo.issues";
  params: { repo: RepoInfo; issues: Issue[] };
}

export interface RepoPatchesRoute {
  resource: "repo.patches";
  rid: string;
  status?: "draft" | "open" | "archived" | "merged";
}

export interface LoadedRepoPatchesRoute {
  resource: "repo.patches";
  params: { repo: RepoInfo; patches: Patch[] };
}

export type RepoRoute = RepoIssuesRoute | RepoPatchesRoute;
export type LoadedRepoRoute = LoadedRepoIssuesRoute | LoadedRepoPatchesRoute;

export async function loadPatches(route: RepoRoute): Promise<LoadedRepoRoute> {
  const repo: RepoInfo = await invoke("repo_by_id", {
    rid: route.rid,
  });
  const patches: Patch[] = await invoke("list_patches", {
    rid: route.rid,
    status: route.status,
  });

  return { resource: "repo.patches", params: { repo, patches } };
}

export async function loadIssues(route: RepoRoute): Promise<LoadedRepoRoute> {
  const repo: RepoInfo = await invoke("repo_by_id", {
    rid: route.rid,
  });
  const issues: Issue[] = await invoke("list_issues", {
    rid: route.rid,
    status: route.status,
  });

  return { resource: "repo.issues", params: { repo, issues } };
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
        return { resource: "repo.issues", rid };
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
        return { resource: "repo.patches", rid };
      }
    } else {
      return null;
    }
  } else {
    return null;
  }
}
