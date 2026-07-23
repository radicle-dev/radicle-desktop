import type { Embed } from "@bindings/cob/Embed";
import type { FileDiff } from "@bindings/diff/FileDiff";

import { Buffer } from "buffer";

import { fileDiffPath } from "@app/lib/diffText";
import { invoke } from "@app/lib/invoke";

// The blob to preview: the new side, except for a deletion where only the old
// side exists.
function binaryBlobOid(file: FileDiff): string {
  return file.status === "deleted" ? file.old.oid : file.new.oid;
}

// Resolve a binary file diff to an object URL when its content is an image, or
// `undefined` for any other binary content.
//
// Fetched once per (rid, blob) and cached: diff rows are virtualized, so a
// file's row unmounts and remounts as it scrolls in and out of view. Without
// the cache we would refetch the embed and mint a fresh object URL on every
// remount.
const cache = new Map<string, Promise<string | undefined>>();

export function binaryImageUrl(
  rid: string,
  file: FileDiff,
): Promise<string | undefined> {
  const oid = binaryBlobOid(file);
  const key = `${rid}:${oid}`;
  const existing = cache.get(key);
  if (existing) {
    return existing;
  }
  const url = invoke<Embed>("get_embed", {
    rid,
    name: fileDiffPath(file),
    oid,
  }).then(embed =>
    embed.mimeType?.startsWith("image/")
      ? URL.createObjectURL(new Blob([Buffer.from(embed.content)]))
      : undefined,
  );
  cache.set(key, url);
  return url;
}
