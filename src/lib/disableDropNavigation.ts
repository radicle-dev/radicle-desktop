// Drops are handled by the webview itself, so anything dropped where nothing
// accepts it (a file, a link, an image) would be opened in place of the app.
// Editable fields keep their native drop, which inserts dragged text or links.
export function disableDropNavigation(): void {
  const isNativeTextDrop = (event: DragEvent) =>
    !(event.dataTransfer?.types.includes("Files") ?? false) &&
    event.target instanceof Element &&
    event.target.closest(
      'input, textarea, [contenteditable]:not([contenteditable="false"])',
    ) !== null;

  window.addEventListener("dragover", event => {
    if (
      event.defaultPrevented ||
      isNativeTextDrop(event) ||
      !event.dataTransfer
    ) {
      return;
    }
    event.preventDefault();
    event.dataTransfer.dropEffect = "none";
  });
  window.addEventListener("drop", event => {
    if (!isNativeTextDrop(event)) {
      event.preventDefault();
    }
  });
}
