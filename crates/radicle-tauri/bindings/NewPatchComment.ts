// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { CodeLocation } from "./CodeLocation";

export type NewPatchComment = {
  id: string;
  revision: string;
  body: string;
  replyTo?: string;
  location: CodeLocation | null;
  embeds: { name: string; content: string }[];
};
