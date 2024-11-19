import { invoke } from "@app/lib/invoke";
import { activeRouteStore } from "@app/lib/router";
import { get } from "svelte/store";
import * as router from "@app/lib/router";

let intervalId: ReturnType<typeof setTimeout>;

export function dynamicInterval(callback: () => void, period: number) {
  clearTimeout(intervalId);

  intervalId = setTimeout(() => {
    callback();
    dynamicInterval(callback, period);
  }, period);
}

let lock = false;

export async function checkAuth() {
  try {
    if (lock) {
      return;
    }
    lock = true;
    await invoke("authenticate");
    if (get(activeRouteStore).resource === "authenticationError") {
      window.history.back();
    }
    dynamicInterval(checkAuth, 30_000);
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
  } catch (e: any) {
    if (get(activeRouteStore).resource !== "authenticationError") {
      await router.push({
        resource: "authenticationError",
        params: {
          error: e.err,
          hint: e.hint,
        },
      });
      dynamicInterval(checkAuth, 1000);
    }
  } finally {
    lock = false;
  }
}
