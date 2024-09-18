import { emit } from "@tauri-apps/api/event";
import { listen } from "@tauri-apps/api/event";
import { writable } from "svelte/store";

let interval: ReturnType<typeof setInterval> | undefined = undefined;

type NodeState = "stopped" | "running";
export let nodeState = writable<NodeState>("stopped");

export function subscribeToNodeEvents() {
  void listen("event", event => {
    console.log(event.payload);
  });

  void listen("node_status", event => {
    nodeState.set(event.payload as NodeState);
  });

  if (interval === undefined) {
    void emit("subscribe_events");

    interval = setInterval(() => {
      // In case there is a running subscription this won't launch a new one.
      void emit("subscribe_events");
    }, 10_000);
  }
}
