import { z } from "zod";

// User-configurable diff/view preferences, persisted to localStorage and shared
// across all commit views so they survive navigation between commits.
export const diffOptionsSchema = z.object({
  // Show the changed-files tree sidebar.
  showTree: z.boolean(),
  // Unified (stacked) vs. split (side-by-side) diff layout.
  diffStyle: z.enum(["unified", "split"]),
  // Wrap long lines instead of scrolling horizontally.
  wordWrap: z.boolean(),
  // Change-indicator style: `classic` (+/−), `bars` (gutter bars), or `none`.
  indicators: z.enum(["classic", "bars", "none"]),
  // In-line change highlighting granularity.
  lineDiffType: z.enum(["word-alt", "word", "char", "none"]),
});

export type DiffOptions = z.infer<typeof diffOptionsSchema>;

const KEY = "diffOptions";

const defaults: DiffOptions = {
  showTree: true,
  diffStyle: "unified",
  wordWrap: false,
  indicators: "bars",
  lineDiffType: "word-alt",
};

function load(): DiffOptions {
  const raw = globalThis.localStorage?.getItem(KEY);
  if (!raw) {
    return { ...defaults };
  }
  try {
    // Merge over defaults so a stored blob from an older schema still loads.
    const parsed = diffOptionsSchema.partial().safeParse(JSON.parse(raw));
    if (parsed.success) {
      return { ...defaults, ...parsed.data };
    }
  } catch {
    // Ignore malformed storage and fall back to defaults.
  }
  return { ...defaults };
}

export const diffOptions = $state<DiffOptions>(load());

// Persist on any change. A detached root effect keeps this alive for the whole
// session (the store is a module singleton, so it is never torn down).
$effect.root(() => {
  $effect(() => {
    globalThis.localStorage?.setItem(KEY, JSON.stringify(diffOptions));
  });
});
