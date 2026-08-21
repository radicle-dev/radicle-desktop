import fs from "node:fs";
import path from "node:path";

import { describe, expect, test } from "vitest";

import { parseTeam } from "@app/lib/team";

const fixturesDir = path.resolve(process.cwd(), "schemas/fixtures");

function readFixture(name: string): string {
  return fs.readFileSync(path.join(fixturesDir, name), "utf8");
}

const fixtures = fs
  .readdirSync(fixturesDir)
  .filter(name => name.endsWith(".json"));
const validFixtures = fixtures.filter(name => name.startsWith("valid-"));
const invalidFixtures = fixtures.filter(name => name.startsWith("invalid-"));

describe("parseTeam conformance", () => {
  test.each(validFixtures)("%s validates", name => {
    expect(parseTeam(readFixture(name)).status).toBe("ok");
  });

  test.each(invalidFixtures)("%s is rejected", name => {
    expect(parseTeam(readFixture(name)).status).not.toBe("ok");
  });
});

describe("parseTeam details", () => {
  test("preserves and surfaces unknown fields", () => {
    const result = parseTeam(readFixture("valid-unknown-fields.json"));
    expect(result.status).toBe("ok");
    if (result.status === "ok") {
      expect(result.unknownFields).toEqual(
        expect.arrayContaining(["dns", "nodes"]),
      );
      expect(result.team).toHaveProperty("dns");
      expect(result.team).toHaveProperty("nodes");
    }
  });

  test("refuses an unsupported future version", () => {
    const result = parseTeam(readFixture("invalid-future-version.json"));
    expect(result).toEqual({ status: "unsupported-version", version: 2 });
  });

  test("reports a malformed file as invalid", () => {
    expect(parseTeam(readFixture("invalid-malformed.json")).status).toBe(
      "invalid",
    );
  });

  test("reports unparseable JSON as invalid", () => {
    expect(parseTeam('{"version": 1').status).toBe("invalid");
  });
});
