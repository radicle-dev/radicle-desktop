/**
 * The properties that decide where a character lands, copied onto the mirror
 * so its layout matches the textarea's line for line.
 */
const mirroredProperties = [
  "boxSizing",
  "borderBottomWidth",
  "borderLeftWidth",
  "borderRightWidth",
  "borderTopWidth",
  "fontFamily",
  "fontSize",
  "fontSizeAdjust",
  "fontStretch",
  "fontStyle",
  "fontVariant",
  "fontWeight",
  "letterSpacing",
  "lineHeight",
  "paddingBottom",
  "paddingLeft",
  "paddingRight",
  "paddingTop",
  "tabSize",
  "textIndent",
  "textTransform",
  "wordSpacing",
] as const;

/**
 * Where a caret position sits inside a textarea, relative to the element's own
 * top left corner and before its scroll offset is applied.
 *
 * A textarea gives no way to ask this directly, so the text up to `position`
 * is laid out again in an off-screen div that copies the textarea's metrics
 * and width. The offset of a marker placed at the position is then the
 * caret's offset.
 */
export function caretCoordinates(
  textarea: HTMLTextAreaElement,
  position: number,
): { top: number; left: number; height: number } {
  const style = window.getComputedStyle(textarea);
  const mirror = document.createElement("div");

  mirror.style.position = "absolute";
  mirror.style.top = "0";
  mirror.style.left = "0";
  mirror.style.visibility = "hidden";
  mirror.style.whiteSpace = "pre-wrap";
  mirror.style.wordWrap = "break-word";
  mirror.style.overflow = "hidden";
  // Only the content box wraps text, so the mirror is sized to match it and
  // keeps the textarea's own padding for the offsets to be measured against.
  mirror.style.width = `${textarea.clientWidth}px`;
  mirror.style.height = "auto";
  for (const property of mirroredProperties) {
    mirror.style[property] = style[property];
  }

  // A trailing newline collapses unless something follows it, which would put
  // the marker one line too high.
  mirror.textContent = textarea.value.slice(0, position).replace(/\n$/, "\n ");

  const marker = document.createElement("span");
  // Zero-width content would give the marker no height to report.
  marker.textContent = "​";
  mirror.appendChild(marker);

  document.body.appendChild(mirror);
  const { offsetTop, offsetLeft, offsetHeight } = marker;
  document.body.removeChild(mirror);

  return {
    top: offsetTop,
    left: offsetLeft,
    height: offsetHeight || parseFloat(style.lineHeight) || 0,
  };
}
