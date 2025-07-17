import type { Config } from "@bindings/config/Config";
import type { RepoCount } from "@bindings/repo/RepoCount";
import type { RepoInfo } from "@bindings/repo/RepoInfo";

import z from "zod";

import { invoke } from "@app/lib/invoke";

export type HomeReposTab = "all" | "delegate" | "private" | "contributor";

const homeReposTabSchema = z.union([
  z.literal("all"),
  z.literal("delegate"),
  z.literal("private"),
  z.literal("contributor"),
]);

export interface HomeRoute {
  resource: "home";
  activeTab: HomeReposTab;
}

export interface LoadedHomeRoute {
  resource: "home";
  params: {
    activeTab: HomeReposTab;
    repoCount: RepoCount;
    repos: RepoInfo[];
    config: Config;
    notificationCount: number;
    seededNotReplicated: string[];
  };
}

export async function loadHome(route: HomeRoute): Promise<LoadedHomeRoute> {
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

  const [config, repoCount, repos, notificationCount, seededNotReplicated] =
    await Promise.all([
      invoke<Config>("config"),
      invoke<RepoCount>("repo_count"),
      invoke<RepoInfo[]>("list_repos", { show }),
      invoke<number>("notification_count"),
      invoke<string[]>("seeded_not_replicated"),
    ]);
  return {
    resource: "home",
    params: {
      activeTab: route.activeTab,
      repoCount,
      repos,
      config,
      notificationCount,
      seededNotReplicated,
    },
  };
}

export function homeUrlToRoute(url: URL): HomeRoute | undefined {
  if (url.pathname === "/") {
    return {
      resource: "home",
      activeTab: homeReposTabSchema
        .catch(() => "all" as const)
        .parse(url.searchParams.get("tab")),
    };
  }
}

export function homeRouteToPath(route: HomeRoute): string {
  const searchParams = new URLSearchParams();
  if (route.activeTab !== "all") {
    searchParams.append("tab", route.activeTab);
  }
  return `/?${searchParams.toString()}`;
}
