import type { UnlistenFn } from "@tauri-apps/api/event";

import { listen } from "@tauri-apps/api/event";
import once from "lodash/once";

import { artifactNodeRunning, nodeRunning } from "./events";

export let unlistenNodeEvents: UnlistenFn | undefined = undefined;

export function setUnlistenNodeEvents(unlisten: UnlistenFn) {
  unlistenNodeEvents = unlisten;
}

// Will be called once in the startup of the app. Subscribes to both node
// liveness events and returns a single unlisten that stops both.
export const createEventEmittersOnce = once(async () => {
  const unlistenNode = await listen<boolean>("node_running", event => {
    nodeRunning.set(event.payload);
  });
  const unlistenArtifactNode = await listen<boolean>(
    "artifact_node_running",
    event => {
      artifactNodeRunning.set(event.payload);
    },
  );
  return () => {
    unlistenNode();
    unlistenArtifactNode();
  };
});
