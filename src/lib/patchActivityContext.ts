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

export interface PatchActivitySource {
  /// Pure lookup: derives the counts from data already in hand. Safe to call
  /// from a `$derived`.
  resolve: (publicKey: string) => PatchUserActivity | undefined;
  /// Loads the repo-wide figures the lookup needs for `patchesAuthored` and
  /// `issuesAuthored`. Separate from `resolve` because it fetches, and a
  /// `$derived` is no place to start one; call it from an effect when a card
  /// is actually opened.
  prefetch: () => void;
}

const KEY = Symbol("patch-activity-context");

export function setPatchActivitySource(source: PatchActivitySource) {
  setContext(KEY, source);
}

export function getPatchActivitySource(): PatchActivitySource | undefined {
  return getContext(KEY);
}
