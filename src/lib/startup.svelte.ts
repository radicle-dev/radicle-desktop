import type { SyncStatus } from "@bindings/repo/SyncStatus";
import type { UnlistenFn } from "@tauri-apps/api/event";

import { listen } from "@tauri-apps/api/event";
import { SvelteMap } from "svelte/reactivity";
import { nodeRunning, syncStatus } from "./events";

export const unlistenEvents: { fn?: UnlistenFn } = $state({ fn: undefined });
export const unlistenNodeEvents: { fn?: UnlistenFn } = $state({
  fn: undefined,
});
export const unlistenSyncStatus: { fn?: UnlistenFn } = $state({
  fn: undefined,
});

export async function createEventEmitters() {
  unlistenEvents.fn = await listen("event", () => {
    // Add handler for incoming events
  });

  unlistenSyncStatus.fn = await listen<Record<string, SyncStatus>>(
    "sync_status",
    event => {
      syncStatus.set(new SvelteMap(Object.entries(event.payload)));
    },
  );

  unlistenNodeEvents.fn = await listen<boolean>("node_running", event => {
    nodeRunning.set(event.payload);
  });
}
