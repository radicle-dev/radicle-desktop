import { repoRouteToPath, repoUrlToRoute } from "@app/views/repo/router";
import { on } from "svelte/events";
import { get, writable } from "svelte/store";

import * as mutexExecutor from "@app/lib/mutexExecutor";
import type { LoadedRoute, Route } from "@app/lib/router/definitions";
import { loadRoute } from "@app/lib/router/definitions";
import * as utils from "@app/lib/utils";

export { type Route };

const InitialStore = { resource: "booting" as const };

export const isLoading = writable<boolean>(true);
export const activeRouteStore = writable<LoadedRoute>(InitialStore);
export const activeUnloadedRouteStore = writable<Route>(InitialStore);

let currentUrl: URL | undefined;

// Where we are in the session history, and how far it goes.
//
// The web view offers neither: `history.length` counts entries the app never
// created (it is already 2 on the first paint) and says nothing about which
// one is current. So each entry the app pushes is stamped with its own
// position, that stamp is read back on every navigation, and the highest
// position seen is remembered as the top of the stack. Entries the app did
// not stamp count as the bottom.
const HISTORY_INDEX = "radicleHistoryIndex";

let historyDepth = 0;

export const canGoBack = writable<boolean>(false);
export const canGoForward = writable<boolean>(false);

function historyIndex(): number {
  const state: unknown = window.history.state;
  const index =
    state && typeof state === "object"
      ? (state as Record<string, unknown>)[HISTORY_INDEX]
      : undefined;
  return typeof index === "number" ? index : 0;
}

function stamp(route: Route, index: number): unknown {
  return { ...route, [HISTORY_INDEX]: index };
}

// Whether Back and Forward have anywhere to go.
//
// `historyDepth` only grows, except on a push, which discards whatever was
// ahead of the entry being pushed onto. Reloading the app forgets it, so for
// the rest of that session Forward stays off until something is pushed again
// — the entries are still there, but nothing in the web view will admit to
// how many.
function syncHistoryBounds(index: number, truncated = false): void {
  historyDepth = truncated ? index : Math.max(historyDepth, index);
  canGoBack.set(index > 0);
  canGoForward.set(index < historyDepth);
}

// Set while a back/forward (popstate) navigation is in flight, then frozen onto
// `historyNavigation` for the route that becomes active, so views can choose to
// restore prior scroll state only for history navigations.
let pendingHistoryNavigation = false;
let historyNavigation = false;

export function isHistoryNavigation(): boolean {
  return historyNavigation;
}

export async function loadFromLocation(): Promise<void> {
  await navigateToUrl("replace", new URL(window.location.href));
}

async function navigateToUrl(
  action: "push" | "replace",
  url: URL,
): Promise<void> {
  const { pathname, hash } = url;

  if (url.origin !== window.origin) {
    throw new Error("Cannot navigate to other origin");
  }

  if (
    currentUrl &&
    currentUrl.pathname === pathname &&
    currentUrl.search === url.search
  ) {
    // The navigation this flag was set for ends here; without the reset a
    // same-URL popstate would leak it onto the next unrelated navigation,
    // which would then wrongly restore cached list state.
    pendingHistoryNavigation = false;
    // The route did not change but our position in the history may have, so
    // the buttons still need re-evaluating.
    syncHistoryBounds(historyIndex());
    return;
  }

  const relativeUrl = pathname + url.search + (hash || "");
  url = new URL(relativeUrl, window.origin);
  const route = urlToRoute(url);

  if (route) {
    await navigate(action, route);
  } else {
    console.error("Could not resolve route for URL: ", url);
    await navigate(action, { resource: "inbox" });
  }
}

const offPopstate = on(window, "popstate", () => {
  pendingHistoryNavigation = true;
  return loadFromLocation();
});

const offNavigateAnchor = on(document, "click", e => {
  const [anchor] = e
    .composedPath()
    .flatMap(target => (target instanceof HTMLAnchorElement ? [target] : []));
  if (anchor && anchor.getAttribute("href")?.startsWith("/")) {
    e.preventDefault();
    void navigateToUrl(
      "push",
      new URL(anchor.getAttribute("href") ?? "", window.location.href),
    );
  }
});

if (import.meta.hot) {
  import.meta.hot.dispose(() => {
    offPopstate();
    offNavigateAnchor();
  });
}

const loadExecutor = mutexExecutor.create();

async function navigate(
  action: "push" | "replace",
  newRoute: Route,
): Promise<void> {
  const historyNav = pendingHistoryNavigation;
  pendingHistoryNavigation = false;
  isLoading.set(true);
  const path = routeToPath(newRoute);

  if (
    action === "push" &&
    path !== window.location.pathname + window.location.search
  ) {
    // Pushing the route that is already active would mint a duplicate
    // history entry, making Back appear to do nothing.
    const index = historyIndex() + 1;
    window.history.pushState(stamp(newRoute, index), "", path);
    syncHistoryBounds(index, true);
  } else if (action === "replace") {
    // A replace stays where it is, so it keeps the index it is overwriting.
    const index = historyIndex();
    window.history.replaceState(stamp(newRoute, index), "");
    syncHistoryBounds(index);
  } else {
    syncHistoryBounds(historyIndex());
  }
  currentUrl = new URL(window.location.href);
  const currentLoadedRoute = get(activeRouteStore);

  const loadedRoute = await loadExecutor.run(async () => {
    return loadRoute(newRoute, currentLoadedRoute);
  });

  // Only let the last request through.
  if (loadedRoute === undefined) {
    return;
  }

  historyNavigation = historyNav;
  activeRouteStore.set(loadedRoute);
  activeUnloadedRouteStore.set(newRoute);
  isLoading.set(false);
  Array.from(
    document.getElementsByClassName("global-reset-scroll-after-navigate"),
  ).forEach(el => {
    el.scrollTo(0, 0);
  });
}

export async function push(newRoute: Route): Promise<void> {
  await navigate("push", newRoute);
}

export async function replace(newRoute: Route): Promise<void> {
  await navigate("replace", newRoute);
}

function inboxUrlToRoute(url: URL): { resource: "inbox" } | undefined {
  if (url.pathname === "/inbox") {
    return { resource: "inbox" };
  }
}

function guideUrlToRoute(url: URL): { resource: "guide" } | undefined {
  if (url.pathname === "/guide") {
    return { resource: "guide" };
  }
}

function urlToRoute(url: URL): Route | null {
  const segments = url.pathname.substring(1).split("/");
  const resource = segments.shift();

  if (url.pathname === "/") {
    return { resource: "inbox" };
  }

  const inboxRoute = inboxUrlToRoute(url);
  if (inboxRoute) {
    return inboxRoute;
  }

  const guideRoute = guideUrlToRoute(url);
  if (guideRoute) {
    return guideRoute;
  }

  switch (resource) {
    case "repos": {
      return repoUrlToRoute(segments, url.searchParams);
    }
    default: {
      return null;
    }
  }
}

export function routeToPath(route: Route): string {
  if (route.resource === "inbox") {
    return "/inbox";
  } else if (route.resource === "guide") {
    return "/guide";
  } else if (
    route.resource === "repo.home" ||
    route.resource === "repo.commits" ||
    route.resource === "repo.commit" ||
    route.resource === "repo.issue" ||
    route.resource === "repo.issues" ||
    route.resource === "repo.patch" ||
    route.resource === "repo.patches"
  ) {
    return repoRouteToPath(route);
  } else if (route.resource === "booting") {
    return "";
  } else {
    return utils.unreachable(route);
  }
}
