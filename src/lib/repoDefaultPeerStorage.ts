import { record, string } from "zod";

import useLocalStorage from "@app/lib/useLocalStorage.svelte";

const storage = useLocalStorage(
  "repoDefaultPeers",
  record(string(), string()),
  {},
  !window.localStorage,
);

export function getDefaultPeer(rid: string): string | undefined {
  return storage.value[rid];
}

export function setDefaultPeer(rid: string, peer: string): void {
  storage.update(current => ({ ...current, [rid]: peer }));
}

export function clearDefaultPeer(rid: string): void {
  storage.update(current => {
    const { [rid]: _removed, ...rest } = current;
    return rest;
  });
}

export function isDefaultPeer(rid: string, peer: string): boolean {
  return storage.value[rid] === peer;
}
