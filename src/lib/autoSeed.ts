import { writable } from "svelte/store";

const KEY = "autoSeed";

function load(): boolean {
  const stored = localStorage ? localStorage.getItem(KEY) : null;
  // Seeding what you download is the neighbourly default, and it is what
  // keeps an artifact reachable once its author goes offline.
  return stored === null ? true : stored === "true";
}

export const autoSeed = writable<boolean>(load());

export function storeAutoSeed(value: boolean): void {
  autoSeed.set(value);
  if (localStorage) {
    localStorage.setItem(KEY, value.toString());
  } else {
    console.warn(
      "localStorage isn't available, not able to persist the auto-seed preference without it.",
    );
  }
}
