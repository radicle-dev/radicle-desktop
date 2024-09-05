import type { RepoInfo } from "@bindings/RepoInfo";
import type { Config } from "@bindings/Config";

import { invoke } from "@tauri-apps/api/core";

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

interface DesignSystemRoute {
  resource: "designSystem";
}

export type Route =
  | BootingRoute
  | DesignSystemRoute
  | HomeRoute
  | AuthenticationErrorRoute;

export type LoadedRoute =
  | BootingRoute
  | DesignSystemRoute
  | LoadedHomeRoute
  | AuthenticationErrorRoute;

export async function loadRoute(
  route: Route,
  _previousLoaded: LoadedRoute,
): Promise<LoadedRoute> {
  if (route.resource === "home") {
    const repos: RepoInfo[] = await invoke("list_repos");
    const config: Config = await invoke("config");
    return { resource: "home", params: { repos, config } };
  }
  return route;
}
