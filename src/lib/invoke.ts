import * as tauri from "@tauri-apps/api/core";

export async function invoke<T = null>(
  cmd: string,
  args?: tauri.InvokeArgs,
  options?: tauri.InvokeOptions,
): Promise<T> {
  if (window.__TAURI_INTERNALS__) {
    return tauri.invoke(cmd, args, options);
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
