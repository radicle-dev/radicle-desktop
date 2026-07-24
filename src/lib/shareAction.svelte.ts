import { z } from "zod";

// The action the share widget performs when its main button is clicked.
// Persisted to localStorage and shared across all ShareButton instances so the
// user's preference is remembered app-wide.
export const shareActionSchema = z.enum(["open", "copyLink", "copyId"]);

export type ShareAction = z.infer<typeof shareActionSchema>;

const KEY = "shareAction";

export const defaultShareAction: ShareAction = "copyLink";

function load(): ShareAction {
  const raw = globalThis.localStorage?.getItem(KEY);
  if (!raw) {
    return defaultShareAction;
  }
  const parsed = shareActionSchema.safeParse(raw);
  return parsed.success ? parsed.data : defaultShareAction;
}

export const shareAction = $state<{ value: ShareAction }>({ value: load() });

// Persist on any change. A detached root effect keeps this alive for the whole
// session (the store is a module singleton, so it is never torn down).
$effect.root(() => {
  $effect(() => {
    globalThis.localStorage?.setItem(KEY, shareAction.value);
  });
});
