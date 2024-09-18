import { writable } from "svelte/store";

export const nodeRunning = writable<boolean>(false);
