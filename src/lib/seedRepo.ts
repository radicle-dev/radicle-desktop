import { parseRepositoryId } from "@app/lib/utils";

// Validates a RID typed into the "Seed a repo" form. The form accepts a bare
// `z…` id as well as `rad:z…`, so the result is the RID in the `rad:` form the
// seeded and queued lists use.
export function seedTarget(
  input: string,
  queued: string[],
  seeded: string[],
): { rid: string } | { error: string } {
  const parsed = parseRepositoryId(input.trim());
  if (parsed === undefined) {
    return { error: "RID is not valid" };
  }
  const rid = `${parsed.prefix}${parsed.pubkey}`;
  if (queued.includes(rid)) {
    return { error: "This repo is already queued for fetching" };
  }
  if (seeded.includes(rid)) {
    return { error: "This repo is already seeded" };
  }
  return { rid };
}
