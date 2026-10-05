import type { RepoRoute } from "@app/views/repo/router";
import type { Config } from "@bindings/config/Config";

import { cachedConfig, cachedRepoById } from "@app/lib/invoke";
import { referenceRoute } from "@app/lib/mentions";
import { show } from "@app/lib/modal";
import type { RadReference, RadUri } from "@app/lib/radUri";
import { parseRadUri } from "@app/lib/radUri";
import * as router from "@app/lib/router";
import { explorerHost, explorerLink } from "@app/lib/utils";

import OpenInBrowser from "@app/modals/OpenInBrowser.svelte";

export type DeepLinkTarget =
  { type: "route"; route: RepoRoute } | { type: "external"; url: string };

export function parseDeepLink(
  link: string,
): { type: "uri"; uri: RadUri } | undefined {
  const uri = parseRadUri(link.trim());

  return uri && { type: "uri", uri };
}

export function deepLinkTarget(
  reference: RadReference,
  local: boolean,
  config: Config,
): DeepLinkTarget | undefined {
  const route = local ? referenceRoute(reference) : undefined;
  if (route) return { type: "route", route };

  const url = explorerLink(reference, config);

  return url ? { type: "external", url } : undefined;
}

// Links can arrive before the app has authenticated and routed, so they wait
// until it has, and are opened one at a time.
export function createLinkQueue(open: (link: string) => Promise<void>) {
  const pending: string[] = [];
  let ready = false;
  let flushing = false;

  async function flush() {
    if (!ready || flushing) return;
    flushing = true;
    try {
      for (let link = pending.shift(); link; link = pending.shift()) {
        await open(link).catch(console.error);
      }
    } finally {
      flushing = false;
    }
  }

  return {
    receive(links: string[]) {
      pending.push(...links);
      void flush();
    },
    ready() {
      ready = true;
      void flush();
    },
  };
}

// An untrusted link only ever navigates; it never fetches or seeds, and
// leaving the app for the browser needs the user's confirmation.
async function openDeepLink(link: string) {
  const reference = parseDeepLink(link);
  if (!reference) return;

  const [config, repo] = await Promise.all([
    cachedConfig(),
    cachedRepoById(`rad:${reference.uri.repo}`).catch(() => undefined),
  ]);
  const target = deepLinkTarget(reference, Boolean(repo), config);
  if (target?.type === "route") {
    await router.push(target.route);
  } else if (target?.type === "external") {
    show({
      component: OpenInBrowser,
      props: { url: target.url, host: explorerHost(config) },
    });
  }
}

const queue = createLinkQueue(openDeepLink);
const launchKey = "deepLink:launch";

export function deepLinksReady() {
  queue.ready();
}

export async function listenForDeepLinks() {
  if (!window.__TAURI_INTERNALS__) return;

  const { getCurrent, onOpenUrl } =
    await import("@tauri-apps/plugin-deep-link");
  await onOpenUrl(links => queue.receive(links));

  // A reload of the webview reports the same launch link again.
  const launched = (await getCurrent()) ?? [];
  const seen = readLaunch();
  if (launched.length > 0 && JSON.stringify(launched) !== seen) {
    writeLaunch(JSON.stringify(launched));
    queue.receive(launched);
  }
}

function readLaunch(): string | null {
  try {
    return sessionStorage.getItem(launchKey);
  } catch {
    return null;
  }
}

function writeLaunch(value: string) {
  try {
    sessionStorage.setItem(launchKey, value);
  } catch {
    return;
  }
}
