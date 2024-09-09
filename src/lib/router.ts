import type { LoadedRoute, Route } from "@app/lib/router/definitions";

import { get, writable } from "svelte/store";

import * as mutexExecutor from "@app/lib/mutexExecutor";
import * as utils from "@app/lib/utils";
import { loadRoute } from "@app/lib/router/definitions";

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
    await navigate(action, {
      // On 404 we redirect to the Home page, shouldn't happen.
      resource: "home",
    });
  }
}

window.addEventListener("popstate", () => loadFromLocation());

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
}

export async function push(newRoute: Route): Promise<void> {
  await navigate("push", newRoute);
}

export async function replace(newRoute: Route): Promise<void> {
  await navigate("replace", newRoute);
}

function urlToRoute(url: URL): Route | null {
  const segments = url.pathname.substring(1).split("/");

  const resource = segments.shift();
  switch (resource) {
    case "": {
      return { resource: "home" };
    }
    case "authenticationError": {
      return { resource: "authenticationError", params: { error: "" } };
    }
    default: {
      return null;
    }
  }
}

export function routeToPath(route: Route): string {
  if (route.resource === "home") {
    return "/";
  } else if (route.resource === "authenticationError") {
    return "/authenticationError";
  } else if (route.resource === "booting") {
    return "";
  } else {
    return utils.unreachable(route);
  }
}
