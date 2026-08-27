import { pastedLinkUrl } from "@app/lib/utils";

export type MarkdownFormat = "bold" | "italic" | "code" | "link";

// A single contiguous replacement, so it can be applied through
// `execCommand("insertText")` and stay on the textarea's native undo stack.
export interface TextEdit {
  // The range of the original value to replace.
  from: number;
  to: number;
  text: string;
  // Where the selection lands afterwards, as offsets into the new value.
  selectionStart: number;
  selectionEnd: number;
}

const markers: Record<"bold" | "italic" | "code", string> = {
  bold: "**",
  italic: "_",
  code: "`",
};

const fence = "```";

export function applyTextEdit(value: string, edit: TextEdit): string {
  return value
    .substring(0, edit.from)
    .concat(edit.text, value.substring(edit.to));
}

// Wraps the selection in `marker`, or strips the markers when they are already
// there, so that the shortcut toggles.
function toggleWrap(
  value: string,
  start: number,
  end: number,
  marker: string,
): TextEdit {
  const width = marker.length;
  const selected = value.substring(start, end);

  // The selection covers the markers too, e.g. all of `**bold**`.
  if (
    selected.length >= width * 2 &&
    selected.startsWith(marker) &&
    selected.endsWith(marker)
  ) {
    const inner = selected.substring(width, selected.length - width);
    return {
      from: start,
      to: end,
      text: inner,
      selectionStart: start,
      selectionEnd: start + inner.length,
    };
  }

  // The markers sit just outside the selection, e.g. `**|bold|**`.
  if (
    start >= width &&
    value.substring(start - width, start) === marker &&
    value.substring(end, end + width) === marker
  ) {
    return {
      from: start - width,
      to: end + width,
      text: selected,
      selectionStart: start - width,
      selectionEnd: end - width,
    };
  }

  return {
    from: start,
    to: end,
    text: marker.concat(selected, marker),
    selectionStart: start + width,
    selectionEnd: end + width,
  };
}

// Inline code for a single line, a fenced block once the selection spans more
// than one, which is what the markdown would have to be anyway.
function toggleCode(value: string, start: number, end: number): TextEdit {
  const selected = value.substring(start, end);
  if (!selected.includes("\n")) {
    return toggleWrap(value, start, end, markers.code);
  }

  const opening = `${fence}\n`;
  const closing = `\n${fence}`;

  // The selection covers the fences too.
  if (selected.startsWith(opening) && selected.endsWith(closing)) {
    const inner = selected.substring(
      opening.length,
      selected.length - closing.length,
    );
    return {
      from: start,
      to: end,
      text: inner,
      selectionStart: start,
      selectionEnd: start + inner.length,
    };
  }

  // The fences sit just outside the selection.
  if (
    start >= opening.length &&
    value.substring(start - opening.length, start) === opening &&
    value.substring(end, end + closing.length) === closing
  ) {
    return {
      from: start - opening.length,
      to: end + closing.length,
      text: selected,
      selectionStart: start - opening.length,
      selectionEnd: end - opening.length,
    };
  }

  return {
    from: start,
    to: end,
    text: opening.concat(selected, closing),
    selectionStart: start + opening.length,
    selectionEnd: start + opening.length + selected.length,
  };
}

// Leaves the caret in whichever half of the link is still empty: the target for
// a selected caption, the caption when the selection is already a URL.
function insertLink(value: string, start: number, end: number): TextEdit {
  const selected = value.substring(start, end);
  const url = pastedLinkUrl(selected);
  const text = url ? `[](${url})` : `[${selected}]()`;
  const caret = url ? start + 1 : start + text.length - 1;

  return {
    from: start,
    to: end,
    text,
    selectionStart: caret,
    selectionEnd: caret,
  };
}

export function applyMarkdownFormat(
  format: MarkdownFormat,
  value: string,
  start: number,
  end: number,
): TextEdit {
  switch (format) {
    case "bold":
      return toggleWrap(value, start, end, markers.bold);
    case "italic":
      return toggleWrap(value, start, end, markers.italic);
    case "code":
      return toggleCode(value, start, end);
    case "link":
      return insertLink(value, start, end);
  }
}

// Pasting a URL over a selection turns the selection into the link caption.
// The caret ends up after the link, where typing would continue.
export function pasteLinkEdit(
  value: string,
  start: number,
  end: number,
  url: string,
): TextEdit {
  const text = `[${value.substring(start, end)}](${url})`;
  const caret = start + text.length;

  return {
    from: start,
    to: end,
    text,
    selectionStart: caret,
    selectionEnd: caret,
  };
}
