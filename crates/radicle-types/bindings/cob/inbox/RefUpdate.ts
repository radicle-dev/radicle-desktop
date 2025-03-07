// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.

export type RefUpdate =
  | { "type": "updated"; name: string; old: string; new: string }
  | { "type": "created"; name: string; oid: string }
  | { "type": "deleted"; name: string; oid: string }
  | { "type": "skipped"; name: string; oid: string };
