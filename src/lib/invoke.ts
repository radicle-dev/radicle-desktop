import * as tauri from "@tauri-apps/api/core";

export async function invoke<T = null>(
  cmd: string,
  args?: tauri.InvokeArgs,
  options?: tauri.InvokeOptions,
): Promise<T> {
  if (window.__TAURI_INTERNALS__) {
    return tauri.invoke(cmd, args, options);
  } else {
    return fetch(`http://127.0.0.1:8080/${cmd}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(args),
    })
      .then(data => data.json())
      .catch(() => {
        throw Error(`Issue with HTTP Route: ${JSON.stringify({ cmd, args })}`);
      });
  }
}
