// THIS FILE IS AUTO-GENERATED. DO NOT EDIT MANUALLY.

import type { ErrorDiscriminants } from "./ErrorDiscriminants";

import type { CryptoErrorDiscriminants } from "./CryptoErrorDiscriminants";
import type { EmbedsErrorDiscriminants } from "./EmbedsErrorDiscriminants";
import type { GitErrorDiscriminants } from "./GitErrorDiscriminants";
import type { IdentityErrorDiscriminants } from "./IdentityErrorDiscriminants";
import type { InboxErrorDiscriminants } from "./InboxErrorDiscriminants";
import type { IssueErrorDiscriminants } from "./IssueErrorDiscriminants";
import type { NodeErrorDiscriminants } from "./NodeErrorDiscriminants";
import type { PatchErrorDiscriminants } from "./PatchErrorDiscriminants";
import type { RepoErrorDiscriminants } from "./RepoErrorDiscriminants";

export type Error =
  | `${"CryptoError"}.${CryptoErrorDiscriminants}`
  | `${"EmbedsError"}.${EmbedsErrorDiscriminants}`
  | `${"GitError"}.${GitErrorDiscriminants}`
  | `${"IdentityError"}.${IdentityErrorDiscriminants}`
  | `${"InboxError"}.${InboxErrorDiscriminants}`
  | `${"IssueError"}.${IssueErrorDiscriminants}`
  | `${"NodeError"}.${NodeErrorDiscriminants}`
  | `${"PatchError"}.${PatchErrorDiscriminants}`
  | `${"RepoError"}.${RepoErrorDiscriminants}`
  | Exclude<ErrorDiscriminants, "CryptoError">
  | Exclude<ErrorDiscriminants, "EmbedsError">
  | Exclude<ErrorDiscriminants, "GitError">
  | Exclude<ErrorDiscriminants, "IdentityError">
  | Exclude<ErrorDiscriminants, "InboxError">
  | Exclude<ErrorDiscriminants, "IssueError">
  | Exclude<ErrorDiscriminants, "NodeError">
  | Exclude<ErrorDiscriminants, "PatchError">
  | Exclude<ErrorDiscriminants, "RepoError">

export type ErrorWrapper = { type: Error; message: string };
