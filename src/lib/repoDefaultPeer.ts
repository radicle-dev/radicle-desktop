import type { Route } from "@app/lib/router/definitions";
import type {
  RepoCommitsRoute,
  RepoHomeRoute,
} from "@app/views/repo/router";

import { getDefaultPeer } from "@app/lib/repoDefaultPeerStorage";

type SourceRoute = RepoHomeRoute | RepoCommitsRoute;

function isSourceRoute(route: Route): route is SourceRoute {
  return route.resource === "repo.home" || route.resource === "repo.commits";
}

export function applyDefaultPeerView(route: Route): Route {
  if (!isSourceRoute(route)) {
    return route;
  }
  if (route.peer !== undefined || route.canonical) {
    return route;
  }

  const defaultPeer = getDefaultPeer(route.rid);
  if (defaultPeer === undefined) {
    return route;
  }

  return { ...route, peer: defaultPeer };
}
