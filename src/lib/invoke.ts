import type { Commit } from "@bindings/repo/Commit";
import type { Diff } from "@bindings/diff/Diff";
import type { DiffOptions } from "@bindings/cob/DiffOptions";
import type { Stats } from "@bindings/cob/Stats";

import * as tauri from "@tauri-apps/api/core";
import { cached } from "@app/lib/cached";

export async function invoke<T = null>(
  cmd: string,
  args?: tauri.InvokeArgs,
  options?: tauri.InvokeOptions,
): Promise<T> {
  return withTestBackend<T>(tauri.invoke, cmd, args, options);
}

async function withTestBackend<T>(
  fn: (
    cmd: string,
    args?: tauri.InvokeArgs,
    options?: tauri.InvokeOptions,
  ) => Promise<T>,
  cmd: string,
  args?: tauri.InvokeArgs,
  options?: tauri.InvokeOptions,
) {
  if (window.__TAURI_INTERNALS__) {
    return fn(cmd, args, options);
  } else {
    return fetch(`http://127.0.0.1:8081/${cmd}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(args),
    }).then(async response => {
      if (response.status === 404) {
        console.log("Got a 404 response:", response);
        return null;
      }
      const json = await response.json();
      if (!response.ok) {
        throw json;
      }

      return json;
    });
  }
}

async function getDiff(rid: string, options: DiffOptions): Promise<Diff> {
  return withTestBackend(tauri.invoke, "get_diff", {
    rid,
    options,
  });
}

export const cachedGetDiff = cached(
  getDiff,
  (...[rid, options]) =>
    `get_diff:${rid}:${JSON.stringify(options, Object.keys(options).sort())}`,
  { max: 10_000 },
);

async function listCommits(
  rid: string,
  base: string,
  head: string,
): Promise<Commit[]> {
  return withTestBackend(tauri.invoke, "list_commits", { rid, base, head });
}

export const cachedListCommits = cached(
  listCommits,
  (...[rid, base, head]) => `list_commits:${rid}:${base}:${head}`,
  { max: 5_000 },
);

async function diffStats(
  rid: string,
  base: string,
  head: string,
): Promise<Stats> {
  return withTestBackend(tauri.invoke, "diff_stats", {
    rid,
    base,
    head,
  });
}

export const cachedDiffStats = cached(
  diffStats,
  (...[rid, base, head]) => `diff_stats:${rid}:${base}:${head}`,
  { max: 10_000 },
);

export async function writeToClipboard(
  text: string,
  opts?: {
    label?: string;
  },
) {
  if (window.__TAURI_INTERNALS__) {
    await tauri.invoke("plugin:clipboard-manager|write_text", {
      label: opts?.label,
      text,
    });
  } else {
    await navigator.clipboard.writeText(text);
  }
}
