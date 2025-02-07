import type { SyncStatus } from "@bindings/repo/SyncStatus";

import { SvelteMap } from "svelte/reactivity";
import { writable } from "svelte/store";

export const nodeRunning = writable<boolean>(false);
export const syncStatus = writable<SvelteMap<string, SyncStatus>>(
  new SvelteMap(),
);
