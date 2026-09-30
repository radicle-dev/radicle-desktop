export function basename(path: string): string {
  return path.split("/").pop() ?? path;
}

export function embedPreviewKind(
  mimeType: string | null,
): "image" | "document" | "video" | "audio" | undefined {
  if (mimeType?.startsWith("image")) return "image";
  if (mimeType?.startsWith("application")) return "document";
  if (mimeType?.startsWith("video")) return "video";
  if (mimeType?.startsWith("audio")) return "audio";
  return undefined;
}
