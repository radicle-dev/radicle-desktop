import { emit } from "@tauri-apps/api/event";

let interval: ReturnType<typeof setInterval> | undefined = undefined;

export function subscribeToNodeEvents() {
  if (interval === undefined) {
    interval = setInterval(() => {
      // In case there is a running subscription this won't launch a new one.
      void emit("subscribe_events");
    }, 10_000);
  }
}
