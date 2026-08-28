# Schemas

## `team.schema.json`

JSON Schema (draft 2020-12) for `.radicle/team.json`, the file that declares a
Radicle team. A repository is a team if and only if that file exists in the root
tree of its default branch. One repository declares exactly one team, and the
repository's RID is the team's identifier.

This file is a **copy of a convention that is normative elsewhere**. It is kept
here so the app can validate without a network fetch, and so this repo is
self-contained. The convention is the Teams RIP, currently the draft
`kTbz-teams` in the `rips` repository. Keep this copy byte-identical to
`kTbz-teams/data/team.schema.json` there; `src/lib/team.ts` reads its identifier
patterns straight out of this file so the parser cannot drift from it.

Consumers other than this app read the same file, so the app does not own the
format. In particular, `additionalProperties` is `true` on purpose: unknown
fields are legal, must be preserved on rewrite, and should be surfaced to the
user rather than hidden.

### Fixtures

`fixtures/` holds conformance vectors. Files named `valid-*` must validate,
files named `invalid-*` must not.

| File | Exercises |
|---|---|
| `valid-typical.json` | Two members, two repos, description present. |
| `valid-minimal.json` | Empty member and repo lists, no description. A newly created team. |
| `valid-unknown-fields.json` | Extra `dns` and `nodes` fields written by another tool. Must validate, and must survive a round-trip. |
| `valid-unresolvable-refs.json` | Well formed DIDs and RIDs that this node has never seen, so nothing resolves to a name. |
| `invalid-malformed.json` | Empty `name`, a member that is not a DID, a duplicate member, an RID missing its prefix. |
| `invalid-future-version.json` | `version: 2`. Must be refused rather than partially parsed. |
| `invalid-wrong-multicodec.json` | A well formed x25519 `did:key`. Rejected because its encoding begins `z6LS`, not `z6Mk`. |

`fixtures/profile-README.md` is sample content for `.radicle/profile/README.md`,
including a relative image link. Note that relative images in rendered markdown
do not currently resolve in this app.

## `teams-payload.schema.json`

Schema for the value of the `dev.radicle.teams.v1` payload in a repository's
identity document — the optional reverse assertion. A team lists the
repositories it covers; a repository may assert the reverse by adding this
payload to its identity document (an identity revision, agreed by its
delegates), not by publishing a file on its default branch.

```json
{
  "teams": ["rad:z3gqcJUoA1n9HaHKufZs5FCSGazv5"]
}
```

The payload is optional and **its absence carries no meaning**. Changing the
identity document requires a delegate majority, so most associations will be
one-sided for a long time. Treat corroboration as a positive signal when
present and show nothing when absent. Never warn on a one-sided association: a
signal that fires on the common case teaches people to ignore it.

`team.json` (a file on the default branch) means the repository *is* a team; the
`dev.radicle.teams.v1` payload means it belongs to one. A repository may hold both,
which is a team that itself belongs to another team.

The payload carries **no `version` field**: the identifier carries the version,
so `dev.radicle.teams.v1` is itself the compatibility boundary and a future
incompatible shape is published as `dev.radicle.teams.v2`.

There are four states per listed repository, and the fourth is not the second:

| Team lists repo | Repo lists team | State |
|---|---|---|
| yes | yes | both assert |
| yes | no | team asserts only |
| no | yes | repo asserts only |
| yes | not replicated locally | unknown |

Reading the reverse assertion requires that repository's identity document, so
for an RID this node has not replicated the answer is unknown rather than absent.

### Fixtures

In `fixtures/teams-payload/`, kept in a subfolder deliberately: the team fixture
tests discover `fixtures/*.json` by prefix and assert every `valid-*` parses as a
team, so a payload file sitting beside them would break that suite.

| File | Conforms | Exercises |
|---|---|---|
| `valid-payload-single.json` | yes | One team. |
| `valid-payload-empty.json` | yes | Empty list, asserting no affiliation. |
| `invalid-payload-did.json` | no | A DID where an RID is expected. |
| `invalid-payload-oversize-rid.json` | no | An identifier longer than any object id encodes to. |

### Notes

- `version` gates interpretation. A file declaring an unknown version must be
  refused, not read for the fields that happen to look familiar.
- `members` and `repos` are required but may be empty.
- The `rid` definition derives from heartwood's published `RepoId` schema (what
  `rad config schema` outputs), anchored and bounded here where upstream's is
  neither. The bound admits 20 to 28 characters after the multibase `z`, the
  full range of a 20 byte SHA-1 object id: 28 at most, and shorter as leading
  zero bytes accumulate. That is the only object format Radicle defines, so the
  CID-wrapped forms proposed in rips patch 31cfefd are excluded until that
  proposal lands.
- Both patterns are lexical filters, not decoders. A value can match and still
  fail to decode, so validate for shape and decode for certainty.
- The `did` pattern is fixed at `z6Mk` plus 44 characters, because the encoded
  payload is always the 2 byte ed25519-pub multicodec plus a 32 byte key, which
  always encodes to 48 characters beginning `z6Mk`. Other key types encode to a
  different prefix and are rejected.
- Canonical form only. Heartwood's parsers also accept bare RIDs, `rad://`
  URLs, and DIDs in other multibase encodings, but nothing ever writes those.
