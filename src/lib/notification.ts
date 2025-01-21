import type { ActionWithAuthor } from "@bindings/cob/inbox/ActionWithAuthor";
import type { Action as IssueAction } from "@bindings/cob/issue/Action";
import type { Action as PatchAction } from "@bindings/cob/patch/Action";

import { pluralize, formatOid } from "@app/lib/utils";

export type Action =
  | ActionWithAuthor<IssueAction>
  | ActionWithAuthor<PatchAction>;

// N.b. I have taken the `%` char as indicator for a `global-oid` class
export function createSummary(
  a: Action[],
  kind: "issue" | "patch",
  oid: string,
  count: number,
) {
  const lastAction = a[a.length - 1];
  let summary = `${lastAction.type} not implemented!`;

  function times(count: number) {
    return count > 1 ? `${count} times` : "";
  }

  if (lastAction.oid === oid) {
    summary = `opened ${kind} %${formatOid(lastAction.oid)}%`;
  } else if (lastAction.type === "comment") {
    summary = `left ${count > 1 ? count : "a"} ${pluralize("comment", count)}`;
  } else if (lastAction.type === "revision") {
    const revisions = a.map(i => `%${formatOid(i.oid)}%`).slice(0, 10);
    summary = `created ${pluralize("revision", count)} ${[
      ...revisions,
      ...(a.length >= 11 ? ["%…%"] : []),
    ].join(", ")}`;
  } else if (lastAction.type === "merge") {
    summary = `merged ${pluralize("revision", count)} %${formatOid(lastAction.revision)}%`;
  } else if (lastAction.type === "edit" && kind === "issue") {
    summary = `edited issue${count ? times(count) : ""}`;
  } else if (lastAction.type === "edit" && kind === "patch") {
    summary = `edited ${pluralize("revision", count)} %${formatOid(lastAction.oid)}%`;
  } else if (lastAction.type === "revision.edit") {
    summary = `edited ${pluralize("revision", count)} %${formatOid(lastAction.revision)}%`;
  } else if (lastAction.type === "lifecycle" && count > 1) {
    summary = `changed to ${lastAction.state.status} and ${count} more changes`;
  } else if (
    lastAction.type === "lifecycle" &&
    lastAction.state.status === "draft"
  ) {
    summary = `changed to ${lastAction.state.status}`;
  } else if (
    lastAction.type === "lifecycle" &&
    lastAction.state.status === "open"
  ) {
    summary = `reopened ${kind}`;
  } else if (
    lastAction.type === "lifecycle" &&
    lastAction.state.status === "archived"
  ) {
    summary = `archived ${kind}`;
  } else if (
    lastAction.type === "lifecycle" &&
    lastAction.state.status === "closed" &&
    lastAction.state.reason === "solved"
  ) {
    summary = `closed ${kind} as solved`;
  } else if (
    lastAction.type === "lifecycle" &&
    lastAction.state.status === "closed"
  ) {
    summary = `closed ${kind}`;
  } else if (lastAction.type === "revision.comment") {
    summary = `left ${count > 1 ? count : "a"} review ${pluralize("comment", count)}`;
  } else if (lastAction.type === "review.comment") {
    summary = `left ${count > 1 ? count : "a"} review ${pluralize("comment", count)}`;
  } else if (a.every(e => e.type === "comment.react")) {
    const reactions = a.map(i => i.reaction).slice(0, 10);
    summary = `reacted with ${[
      ...reactions,
      ...(a.length >= 11 ? ["%…%"] : []),
    ].join(", ")} to ${count > 1 ? count : "a"} ${pluralize("comment", count)}`;
  } else if (lastAction.type === "comment.edit") {
    summary = `edited ${count > 1 ? count : "a"} ${pluralize("comment", count)}`;
  } else if (lastAction.type === "review" && lastAction.verdict) {
    summary = `${lastAction.verdict}ed revision %${formatOid(lastAction.revision)}% with a review`;
  } else if (lastAction.type === "review") {
    summary = `left a review with a comment on revision %${formatOid(lastAction.revision)}%`;
  } else if (lastAction.type === "assign") {
    summary = "changed assignes";
  } else if (lastAction.type === "revision.comment.edit") {
    summary = `edited ${count > 1 ? count : "a"} ${pluralize("comment", count)}`;
  } else if (lastAction.type === "comment.redact") {
    summary = `redacted ${count > 1 ? count : "a"} ${pluralize("comment", count)}`;
  } else if (lastAction.type === "label") {
    summary = `added ${pluralize("label", count)}`;
  } else if (lastAction.type === "review.comment.edit") {
    summary = `edited ${count > 1 ? count : "a"} review ${pluralize("comment", count)}`;
  } else if (lastAction.type === "review.comment.react") {
    summary = `reacted to ${count > 1 ? count : "a"} review ${pluralize("comment", count)}`;
  } else if (lastAction.type === "review.comment.redact") {
    summary = `redacted ${count > 1 ? count : "a"} review ${pluralize("comment", count)}`;
  } else if (lastAction.type === "review.comment.resolve") {
    summary = `resolved ${count > 1 ? count : "a"} review ${pluralize("comment", count)}`;
  } else if (lastAction.type === "review.comment.unresolve") {
    summary = `unresolved ${count > 1 ? count : "a"} review ${pluralize("comment", count)}`;
  } else if (lastAction.type === "review.edit") {
    summary = `edited ${count > 1 ? count : "a"} ${pluralize("review", count)}`;
  } else if (lastAction.type === "review.redact") {
    summary = `redacted ${count > 1 ? count : "a"} ${pluralize("review", count)}`;
  } else if (lastAction.type === "revision.comment.react") {
    summary = `reacted to ${count > 1 ? count : "a"} revision ${pluralize("comment", count)}`;
  } else if (lastAction.type === "revision.comment.redact") {
    summary = `redacted ${count > 1 ? count : "a"} revision ${pluralize("comment", count)}`;
  } else if (lastAction.type === "revision.react") {
    summary = `reacted to ${count > 1 ? count : "a"} ${pluralize("revision", count)}`;
  } else if (lastAction.type === "revision.redact") {
    summary = `redacted ${count > 1 ? count : "a"} ${pluralize("revision", count)}`;
  }

  return summary.replaceAll(
    /[%](\S+)[%]/g,
    '<span class="global-oid">$1</span>',
  );
}

export function compressActions(
  actions: Action[],
  kind: "issue" | "patch",
  oid: string,
) {
  const result: {
    summary: string;
    oid: string;
    items: Action[];
  }[] = [];

  let currentGroup: Action[] = [];

  for (const action of actions) {
    if (currentGroup.length === 0) {
      currentGroup.push(action);
      continue;
    }

    const last = currentGroup[currentGroup.length - 1];
    const sameAuthorDid = last.author.did === action.author.did;
    const sameType =
      last.type === action.type ||
      (last.type === "revision.edit" && action.type === "edit") ||
      (action.type === "revision.edit" && last.type === "edit") ||
      action.oid === oid;

    if (sameAuthorDid && sameType) {
      currentGroup.push(action);
    } else {
      const summaryStr = createSummary(
        currentGroup,
        kind,
        oid,
        currentGroup.length,
      );

      result.push({
        summary: summaryStr,
        oid,
        items: currentGroup,
      });

      currentGroup = [action];
    }
  }

  // Summarize any open actions.
  if (currentGroup.length > 0) {
    const summaryStr = createSummary(
      currentGroup,
      kind,
      oid,
      currentGroup.length,
    );
    result.push({
      summary: summaryStr,
      oid,
      items: currentGroup,
    });
  }

  return result;
}
