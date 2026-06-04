import { writable } from "svelte/store";

const KEY = "autoSeedArtifacts";

function loadAutoSeed(): boolean {
  const stored = localStorage ? localStorage.getItem(KEY) : null;
  // Default on: artifacts seed over iroh unless the user opts out.
  return stored === null ? true : stored === "true";
}

export const autoSeedArtifacts = writable<boolean>(loadAutoSeed());

export function storeAutoSeed(value: boolean): void {
  autoSeedArtifacts.set(value);
  if (localStorage) {
    localStorage.setItem(KEY, value.toString());
  } else {
    console.warn(
      "localStorage isn't available, not able to persist the auto-seed preference without it.",
    );
  }
}
