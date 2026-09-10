import { z } from "zod";

import orgJsonSchema from "../../schemas/org.schema.json";

const didPattern = new RegExp(orgJsonSchema.$defs.did.pattern);
const ridPattern = new RegExp(orgJsonSchema.$defs.rid.pattern);

function unique<T>(items: T[]): boolean {
  return new Set(items).size === items.length;
}

export const orgSchema = z.looseObject({
  version: z.literal(1),
  name: z.string().min(1).max(255),
  description: z.string().max(1024).optional(),
  members: z
    .array(
      z.union([z.string().regex(ridPattern), z.string().regex(didPattern)]),
    )
    .max(1024)
    .refine(unique, { message: "Members must be unique" }),
  repos: z
    .array(z.string().regex(ridPattern))
    .max(4096)
    .refine(unique, { message: "Repos must be unique" }),
});

export type Org = z.infer<typeof orgSchema>;

const knownKeys = new Set([
  "version",
  "name",
  "description",
  "members",
  "repos",
]);

export type OrgParseResult =
  | { status: "ok"; org: Org; unknownFields: string[] }
  | { status: "unsupported-version"; version: number }
  | { status: "invalid"; message: string };

export function parseOrg(raw: string): OrgParseResult {
  let value: unknown;
  try {
    value = JSON.parse(raw);
  } catch {
    return { status: "invalid", message: "The file is not valid JSON." };
  }

  if (
    typeof value === "object" &&
    value !== null &&
    "version" in value &&
    typeof (value as { version: unknown }).version === "number"
  ) {
    const { version } = value as { version: number };
    if (Number.isInteger(version) && version > 1) {
      return { status: "unsupported-version", version };
    }
  }

  const result = orgSchema.safeParse(value);
  if (!result.success) {
    return { status: "invalid", message: z.prettifyError(result.error) };
  }

  const unknownFields = Object.keys(result.data).filter(
    key => !knownKeys.has(key),
  );

  return { status: "ok", org: result.data, unknownFields };
}
