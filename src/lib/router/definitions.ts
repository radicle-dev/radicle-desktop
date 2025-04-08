import type { Config } from "@bindings/config/Config";
import type { NotificationItem } from "@bindings/cob/inbox/NotificationItem";
import type { PaginatedQuery } from "@bindings/cob/PaginatedQuery";
import type { NotificationCount } from "@bindings/cob/inbox/NotificationCount";
import type { RepoInfo } from "@bindings/repo/RepoInfo";
import type { RepoCount } from "@bindings/repo/RepoCount";
import type { LoadedRepoRoute, RepoRoute } from "@app/views/repo/router";

import { invoke } from "@app/lib/invoke";
import { SvelteMap } from "svelte/reactivity";

import {
  loadCreateIssue,
  loadIssue,
  loadIssues,
  loadPatch,
  loadPatches,
} from "@app/views/repo/router";

export type HomeReposTab = "delegate" | "private" | "contributor";

export interface HomeInboxTab {
  rid: string;
  name: string;
  count: number;
}

interface BootingRoute {
  resource: "booting";
}

interface HomeRoute {
  resource: "home";
  activeTab?: HomeReposTab;
}

interface InboxRoute {
  resource: "inbox";
  activeTab?: HomeInboxTab;
}

interface LoadedInboxRoute {
  resource: "inbox";
  params: {
    activeTab?: HomeInboxTab;
    repoCount: RepoCount;
    notificationCount: SvelteMap<string, NotificationCount>;
    notifications: SvelteMap<
      string,
      {
        repo: HomeInboxTab;
        items: [string, NotificationItem[]][];
        pagination: { cursor: number; more: boolean };
      }
    >;
    config: Config;
  };
}

interface LoadedHomeRoute {
  resource: "home";
  params: {
    activeTab?: HomeReposTab;
    repoCount: RepoCount;
    notificationCount: Map<string, NotificationCount>;
    repos: RepoInfo[];
    config: Config;
  };
}

export type Route = InboxRoute | BootingRoute | HomeRoute | RepoRoute;

export type LoadedRoute =
  | LoadedInboxRoute
  | BootingRoute
  | LoadedHomeRoute
  | LoadedRepoRoute;

export async function loadRoute(
  route: Route,
  _previousLoaded: LoadedRoute,
): Promise<LoadedRoute> {
  const [count, repoCount, config] = await Promise.all([
    invoke<Record<string, NotificationCount>>("count_notifications_by_repo"),
    invoke<RepoCount>("repo_count"),
    invoke<Config>("config"),
  ]);
  const notificationCount = new SvelteMap(Object.entries(count));
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
        notificationCount,
        repos,
        config,
      },
    };
  } else if (route.resource === "inbox") {
    const notifications: LoadedInboxRoute["params"]["notifications"] =
      new SvelteMap();
    if (route.activeTab) {
      const items = await invoke<
        PaginatedQuery<[string, NotificationItem[]][]>
      >("list_notifications", {
        params: {
          repo: route.activeTab.rid,
        },
      });
      notifications.set(route.activeTab.rid, {
        repo: route.activeTab,
        items: items.content,
        pagination: { cursor: items.cursor, more: items.more },
      });
    } else {
      for (const [rid, item] of notificationCount) {
        const result = await invoke<
          PaginatedQuery<[string, NotificationItem[]][]>
        >("list_notifications", {
          params: {
            repo: rid,
          },
        });
        notifications.set(item.rid, {
          repo: { name: item.name, rid: item.rid, count: item.count },
          items: result.content,
          pagination: { cursor: result.cursor, more: result.more },
        });
      }
    }

    return {
      resource: "inbox",
      params: {
        activeTab: route.activeTab,
        repoCount,
        notifications,
        notificationCount,
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
