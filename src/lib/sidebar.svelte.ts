import { boolean, number } from "zod";

import useLocalStorage from "@app/lib/useLocalStorage.svelte";

export const MIN_SIDEBAR_WIDTH = 12;
export const MAX_SIDEBAR_WIDTH = 30;
export const DEFAULT_SIDEBAR_WIDTH = 16.5;

export const sidebarCollapsed = useLocalStorage(
  "sidebarCollapsed",
  boolean(),
  false,
  !window.localStorage,
);

export const sidebarWidth = useLocalStorage(
  "sidebarWidth",
  number(),
  DEFAULT_SIDEBAR_WIDTH,
  !window.localStorage,
);

// Twice the centre of macOS's traffic lights, which sit on 20px centres
// starting at 20px, so the rail's column of icons shares their axis.
export const RAIL_WIDTH_REM = 5;

// True while the user is dragging the resize edge, so the width transition can
// be suppressed for 1:1 pointer tracking.
export const sidebarResizing = $state({ value: false });

export function toggleSidebar() {
  sidebarCollapsed.value = !sidebarCollapsed.value;
}

export function setSidebarWidth(rem: number) {
  sidebarWidth.value = Math.min(
    MAX_SIDEBAR_WIDTH,
    Math.max(MIN_SIDEBAR_WIDTH, rem),
  );
}
