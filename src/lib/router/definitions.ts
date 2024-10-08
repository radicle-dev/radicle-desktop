import type { Config } from "@bindings/Config";
import type { RepoInfo } from "@bindings/RepoInfo";
import type { LoadedRepoRoute, RepoRoute } from "@app/views/repo/router";

import { invoke } from "@tauri-apps/api/core";

import {
  loadCreateIssue,
  loadIssue,
  loadIssues,
  loadPatch,
  loadPatches,
} from "@app/views/repo/router";

interface BootingRoute {
  resource: "booting";
}

interface AuthenticationErrorRoute {
  resource: "authenticationError";
  params: {
    error: string;
    hint?: string;
  };
}

interface HomeRoute {
  resource: "home";
}

interface LoadedHomeRoute {
  resource: "home";
  params: { repos: RepoInfo[]; config: Config };
}

export type Route =
  | AuthenticationErrorRoute
  | BootingRoute
  | HomeRoute
  | RepoRoute;

export type LoadedRoute =
  | AuthenticationErrorRoute
  | BootingRoute
  | LoadedHomeRoute
  | LoadedRepoRoute;

export async function loadRoute(
  route: Route,
  _previousLoaded: LoadedRoute,
): Promise<LoadedRoute> {
  if (route.resource === "home") {
    const repos: RepoInfo[] = await invoke("list_repos");
    const config: Config = await invoke("config");
    return { resource: "home", params: { repos, config } };
  } else if (route.resource === "repo.issue") {
    return loadIssue(route);
  } else if (route.resource === "repo.createIssue") {
    return loadCreateIssue(route);
  } else if (route.resource === "repo.issues") {
    return loadIssues(route);
  } else if (route.resource === "repo.patch") {
    return loadPatch(route);
  } else if (route.resource === "repo.patches") {
    return loadPatches(route);
  }
  return route;
}
