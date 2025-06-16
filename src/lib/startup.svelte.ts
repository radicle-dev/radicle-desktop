import type { UnlistenFn } from "@tauri-apps/api/event";

import once from "lodash/once";
import { listen } from "@tauri-apps/api/event";

import { nodeRunning } from "./events";

export let unlistenNodeEvents: UnlistenFn | undefined = undefined;

export function setUnlistenNodeEvents(unlisten: UnlistenFn) {
  unlistenNodeEvents = unlisten;
}

// Will be called once in the startup of the app
export const createEventEmittersOnce = once(async () => {
  return await listen<boolean>("node_running", event => {
    nodeRunning.set(event.payload);
  });
});
