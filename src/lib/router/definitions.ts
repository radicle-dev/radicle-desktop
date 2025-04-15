import type { LoadedHomeRoute, HomeRoute } from "@app/views/home/router";
import type { LoadedRepoRoute, RepoRoute } from "@app/views/repo/router";

import { loadHome } from "@app/views/home/router";
import {
  loadCreateIssue,
  loadIssue,
  loadIssues,
  loadPatch,
  loadPatches,
  loadRepoHome,
} from "@app/views/repo/router";

interface BootingRoute {
  resource: "booting";
}

export type Route = BootingRoute | HomeRoute | RepoRoute;
export type LoadedRoute = BootingRoute | LoadedHomeRoute | LoadedRepoRoute;

export async function loadRoute(
  route: Route,
  _previousLoaded: LoadedRoute,
): Promise<LoadedRoute> {
  if (route.resource === "home") {
    return loadHome(route);
  } else if (route.resource === "repo.home") {
    return loadRepoHome(route);
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
