import { getContext, setContext } from "svelte";

export interface PatchUserActivity {
  isAuthor: boolean;
  isDelegate: boolean;
  revisionCount: number;
  commitCount: number;
  reviewCount: number;
  patchesAuthored: number;
  issuesAuthored: number;
}

export type PatchActivityResolver = (
  publicKey: string,
) => PatchUserActivity | undefined;

const KEY = Symbol("patch-activity-context");

export function setPatchActivityResolver(resolver: PatchActivityResolver) {
  setContext(KEY, resolver);
}

export function getPatchActivityResolver(): PatchActivityResolver | undefined {
  return getContext(KEY);
}
