import { describe, expect, test } from "vitest";

import { applyDefaultPeerView } from "@app/lib/repoDefaultPeer";
import {
  clearDefaultPeer,
  getDefaultPeer,
  setDefaultPeer,
} from "@app/lib/repoDefaultPeerStorage";
import { repoRouteToPath } from "@app/views/repo/router";

const RID = "rad:z4D5UCArafTzTQpDZNQRuqswh3ury";
const PEER = "z6MkknknvqZkuDxZ5DtKqy3Ef11wnWFHByfamcoMdN754CG2";

describe("applyDefaultPeerView", () => {
  test("redirects canonical repo home to the stored default peer", () => {
    setDefaultPeer(RID, PEER);
    expect(
      applyDefaultPeerView({ resource: "repo.home", rid: RID }),
    ).toEqual({
      resource: "repo.home",
      rid: RID,
      peer: PEER,
    });
    clearDefaultPeer(RID);
  });

  test("leaves explicit canonical views unchanged", () => {
    setDefaultPeer(RID, PEER);
    expect(
      applyDefaultPeerView({
        resource: "repo.home",
        rid: RID,
        canonical: true,
      }),
    ).toEqual({
      resource: "repo.home",
      rid: RID,
      canonical: true,
    });
    clearDefaultPeer(RID);
  });

  test("does not override an explicit peer", () => {
    setDefaultPeer(RID, PEER);
    const otherPeer = "z6MkkPvBfjP4bQmco7UGsX2ruDBieEHi8n9DVJWX5sTEz";
    expect(
      applyDefaultPeerView({
        resource: "repo.commits",
        rid: RID,
        peer: otherPeer,
      }),
    ).toEqual({
      resource: "repo.commits",
      rid: RID,
      peer: otherPeer,
    });
    clearDefaultPeer(RID);
  });
});

describe("repoRouteToPath canonical flag", () => {
  test("marks explicit canonical home routes in the URL", () => {
    expect(
      repoRouteToPath({
        resource: "repo.home",
        rid: RID,
        canonical: true,
      }),
    ).toBe(`/repos/${RID}/home?canonical=1`);
  });
});

describe("repoDefaultPeerStorage", () => {
  test("stores and clears per-repo defaults", () => {
    expect(getDefaultPeer(RID)).toBeUndefined();
    setDefaultPeer(RID, PEER);
    expect(getDefaultPeer(RID)).toBe(PEER);
    clearDefaultPeer(RID);
    expect(getDefaultPeer(RID)).toBeUndefined();
  });
});
