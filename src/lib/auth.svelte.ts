import type { ErrorWrapper } from "@bindings/error/ErrorWrapper";

import * as router from "@app/lib/router";
import { dynamicInterval } from "@app/lib/interval";
import { get } from "svelte/store";
import { invoke } from "@app/lib/invoke";

export const startup = $state<{ error?: ErrorWrapper }>({ error: undefined });

let lock = false;

export async function checkAuth() {
  try {
    if (lock) {
      return;
    }
    lock = true;
    await invoke("authenticate", { passphrase: "" });
    dynamicInterval(
      "auth",
      checkAuth,
      import.meta.env.VITE_AUTH_LONG_DELAY || 30_000,
    );
    if (get(router.activeRouteStore).resource === "booting") {
      void router.push({ resource: "home" });
    }
  } catch (err) {
    const error = err as ErrorWrapper;
    startup.error = error;
    if (get(router.activeRouteStore).resource !== "booting") {
      void router.push({ resource: "booting" });
    }
    dynamicInterval("auth", checkAuth, 5_000);
  } finally {
    lock = false;
  }
}
