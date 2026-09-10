import fs from "node:fs";
import path from "node:path";

import { describe, expect, test } from "vitest";

import { parseOrg } from "@app/lib/org";

const fixturesDir = path.resolve(process.cwd(), "schemas/fixtures");

function readFixture(name: string): string {
  return fs.readFileSync(path.join(fixturesDir, name), "utf8");
}

const fixtures = fs
  .readdirSync(fixturesDir)
  .filter(name => name.endsWith(".json"));
const validFixtures = fixtures.filter(name => name.startsWith("valid-"));
const invalidFixtures = fixtures.filter(name => name.startsWith("invalid-"));

describe("parseOrg conformance", () => {
  // A conforming file is not always a readable one: a declaration from a
  // later version conforms, and is recognised rather than interpreted.
  test.each(validFixtures)("%s conforms", name => {
    expect(parseOrg(readFixture(name)).status).not.toBe("invalid");
  });

  test.each(invalidFixtures)("%s is rejected", name => {
    expect(parseOrg(readFixture(name)).status).toBe("invalid");
  });

  test("a later version is recognised, not interpreted", () => {
    expect(parseOrg(readFixture("valid-future-version.json")).status).toBe(
      "unsupported-version",
    );
  });
});

describe("parseOrg details", () => {
  test("preserves and surfaces unknown fields", () => {
    const result = parseOrg(readFixture("valid-unknown-fields.json"));
    expect(result.status).toBe("ok");
    if (result.status === "ok") {
      expect(result.unknownFields).toEqual(
        expect.arrayContaining(["dns", "nodes"]),
      );
      expect(result.org).toHaveProperty("dns");
      expect(result.org).toHaveProperty("nodes");
    }
  });

  test("refuses an unsupported future version", () => {
    const result = parseOrg(readFixture("valid-future-version.json"));
    expect(result).toEqual({ status: "unsupported-version", version: 2 });
  });

  test("reports a malformed file as invalid", () => {
    expect(parseOrg(readFixture("invalid-malformed.json")).status).toBe(
      "invalid",
    );
  });

  test("reports unparseable JSON as invalid", () => {
    expect(parseOrg('{"version": 1').status).toBe("invalid");
  });
});
