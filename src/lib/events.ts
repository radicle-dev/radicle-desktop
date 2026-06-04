import { writable } from "svelte/store";

export const nodeRunning = writable<boolean>(false);

// Whether the external rad-artifact node is reachable. Drives the
// seeding/download controls and the sidebar indicator. Polled by the
// backend every 2s via the `artifact_node_running` event.
export const artifactNodeRunning = writable<boolean>(false);
