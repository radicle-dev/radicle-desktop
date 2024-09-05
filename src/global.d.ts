declare global {
  interface Window {
    // eslint-disable-next-line @typescript-eslint/naming-convention
    __TAURI_INTERNALS__: Record<string, unknown>;
  }
}

export {};
