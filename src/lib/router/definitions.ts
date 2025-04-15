import type { Config } from "@bindings/config/Config";
import type { RepoInfo } from "@bindings/repo/RepoInfo";
import type { RepoCount } from "@bindings/repo/RepoCount";
import type { LoadedRepoRoute, RepoRoute } from "@app/views/repo/router";

import { invoke } from "@app/lib/invoke";

import {
  loadCreateIssue,
  loadIssue,
  loadIssues,
  loadPatch,
  loadPatches,
} from "@app/views/repo/router";

export type HomeReposTab = "all" | "delegate" | "private" | "contributor";

interface BootingRoute {
  resource: "booting";
}

interface HomeRoute {
  resource: "home";
  activeTab: HomeReposTab;
}

interface LoadedHomeRoute {
  resource: "home";
  params: {
    activeTab: HomeReposTab;
    repoCount: RepoCount;
    repos: RepoInfo[];
    config: Config;
  };
}

export type Route = BootingRoute | HomeRoute | RepoRoute;

export type LoadedRoute = BootingRoute | LoadedHomeRoute | LoadedRepoRoute;

export async function loadRoute(
  route: Route,
  _previousLoaded: LoadedRoute,
): Promise<LoadedRoute> {
  const [repoCount, config] = await Promise.all([
    invoke<RepoCount>("repo_count"),
    invoke<Config>("config"),
  ]);

  if (route.resource === "home") {
    let show = "all";

    if (route.resource === "home") {
      if (route.activeTab === "delegate") {
        show = "delegate";
      } else if (route.activeTab === "contributor") {
        show = "contributor";
      } else if (route.activeTab === "private") {
        show = "private";
      }
    }

    const repos = await invoke<RepoInfo[]>("list_repos", { show });
    return {
      resource: "home",
      params: {
        activeTab: route.activeTab,
        repoCount,
        repos,
        config,
      },
    };
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
