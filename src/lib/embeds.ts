export function basename(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
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

const pngSignature = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];

function chunkType(bytes: Uint8Array, offset: number): string {
  return String.fromCharCode(...bytes.subarray(offset, offset + 4));
}

export function retinaWidth(bytes: Uint8Array): number | undefined {
  if (
    bytes.length < 24 ||
    !pngSignature.every((byte, i) => bytes[i] === byte) ||
    chunkType(bytes, 12) !== "IHDR"
  ) {
    return undefined;
  }
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const width = view.getUint32(16);

  let offset = 8;
  while (offset + 8 <= bytes.length) {
    const length = view.getUint32(offset);
    const type = chunkType(bytes, offset + 4);
    if (type === "pHYs" && length === 9 && offset + 17 <= bytes.length) {
      const x = view.getUint32(offset + 8);
      const y = view.getUint32(offset + 12);
      const metre = bytes[offset + 16] === 1;
      if (!metre || x !== y) return undefined;
      const scale = Math.round((x * 0.0254) / 72);
      return scale === 2 || scale === 3 ? Math.round(width / scale) : undefined;
    }
    if (type === "IDAT" || type === "IEND") return undefined;
    offset += 12 + length;
  }
  return undefined;
}
