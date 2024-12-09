// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { Addition } from "./Addition";
import type { Deletion } from "./Deletion";
import type { Line } from "../syntax/Line";

export type Modification =
  | { "type": "addition" } & Addition
  | { "type": "deletion" } & Deletion
  | {
    "type": "context";
    line: string;
    lineNoOld: number;
    lineNoNew: number;
    highlight: Line | null;
  };
