import type { SyncStatus } from "@bindings/repo/SyncStatus";

import once from "lodash/once";
import { SvelteMap } from "svelte/reactivity";
import { listen } from "@tauri-apps/api/event";

import { nodeRunning, syncStatus } from "./events";

// Will be called once in the startup of the app
export const createEventEmittersOnce = once(async () => {
  const unlistenEvents = await listen("event", () => {
    // Add handler for incoming events
  });

  const unlistenSyncStatus = await listen<Record<string, SyncStatus>>(
    "sync_status",
    event => {
      syncStatus.set(new SvelteMap(Object.entries(event.payload)));
    },
  );

  const unlistenNodeEvents = await listen<boolean>("node_running", event => {
    nodeRunning.set(event.payload);
  });

  return [unlistenEvents, unlistenSyncStatus, unlistenNodeEvents];
});
