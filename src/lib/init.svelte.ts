import type { Config } from "@bindings/config/Config";
import type { Author } from "@bindings/cob/Author";
import type { SyncStatus } from "@bindings/repo/SyncStatus";
import type { UnlistenFn } from "@tauri-apps/api/event";

import { SvelteMap } from "svelte/reactivity";
import { listen } from "@tauri-apps/api/event";

import { invoke } from "./invoke";
import { nodeRunning, syncStatus } from "./events";
import * as router from "@app/lib/router";

export const GETTING_STARTED_REPO_RID = "rad:z3Wu3dtimpnoMAveMDFGty9UCdv9b";
export const SEED_RADICLE_GARDEN_DID =
  "did:key:z6MkrLMMsiPWUcNPHcRajuMi9mDfYckSoJyPwwnknocNYPm7";
export const SEED_RADICLE_GARDEN_ADDRESS = "seed.radicle.garden:8776";

type Status =
  | { type: "loading" }
  | { type: "missingProfile" }
  | { type: "lockedKeystore"; profile: Author }
  | { type: "creating" };

export const unlistenEvents: { fn: UnlistenFn | undefined } = $state({
  fn: undefined,
});
export const unlistenNodeEvents: { fn: UnlistenFn | undefined } = $state({
  fn: undefined,
});
export const unlistenSyncStatus: { fn: UnlistenFn | undefined } = $state({
  fn: undefined,
});

export const status = $state<Status>({ type: "loading" });

export async function startup(passphrase: string) {
  let profile: undefined | Config;
  try {
    profile = await invoke<Config>("startup");
  } catch (err) {
    console.error(err);
    status.type = "missingProfile";
    return;
  }

  try {
    await invoke("authenticate", { passphrase });
  } catch (err) {
    console.error(err);
    status.type = "lockedKeystore";
    if (status.type === "lockedKeystore" && profile) {
      status.profile = { alias: profile.alias, did: profile.publicKey };
    }
    return;
  }

  if (window.__TAURI_INTERNALS__) {
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

  void router.loadFromLocation();
}
