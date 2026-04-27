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

const offPopstate = on(window, "popstate", () => loadFromLocation());

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
  isLoading.set(true);
  const path = routeToPath(newRoute);

  if (action === "push") {
    window.history.pushState(newRoute, "", path);
  } else if (action === "replace") {
    window.history.replaceState(newRoute, "");
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
