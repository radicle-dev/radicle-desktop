import type { MentionTarget } from "@app/lib/mentions";
import {
  bareOidPattern,
  parseBareIdentifier,
  parseMentionHref,
} from "@app/lib/mentions";

/**
 * The kinds of entity a trigger character offers.
 *
 * `@` searches everything, so a single key reaches people, repos and COBs
 * without having to remember which prefix goes with which. `#` narrows to
 * issues and patches, for when a title would otherwise compete with an alias.
 */
export type MentionScope = "all" | "cobs";

/**
 * What the caret is currently sitting in: either a query being typed after a
 * trigger character, or a complete identifier that can be turned into a link.
 */
export type MentionTrigger =
  | {
      kind: "query";
      scope: MentionScope;
      /** Index of the trigger character in the text. */
      start: number;
      /** Index just past the query, i.e. the caret. */
      end: number;
      query: string;
    }
  | {
      kind: "identifier";
      /** Index the identifier starts at. */
      start: number;
      /** Index just past the identifier, i.e. the caret. */
      end: number;
      target: MentionTarget;
    }
  | {
      kind: "oid";
      start: number;
      end: number;
      /**
       * A full git oid, which on its own does not say what it names. The
       * dropdown resolves it against the current repo, where it may be an
       * issue, a patch or a commit.
       */
      oid: string;
    };

/**
 * The characters a trigger may directly follow. Anything else means the
 * character is part of a word — an email address, or a `#` in a URL fragment —
 * and is left alone.
 */
const openingCharacters = new Set([" ", "\n", "\t", "(", "[", ">", '"', "'"]);

/** Everything up to the caret that a query is allowed to contain. */
const queryPattern = /^[\w.-]*$/;

/**
 * Longest query considered. Past this the user is clearly writing prose rather
 * than picking a reference.
 */
const maximumQueryLength = 64;

/** Where a bare identifier can start within the word the caret is in. */
const identifierPrefixes = ["did:key:", "rad:", "https://", "http://"];

/** Longest identifier considered, generous enough for an explorer URL. */
const maximumIdentifierLength = 256;

/**
 * Find the reference the caret currently sits in, if any: a query being typed
 * after `@` or `#`, or a complete identifier that was typed or pasted whole.
 *
 * Only text between the trigger and the caret is considered, so moving the
 * caret out of a query dismisses the dropdown, and editing an already
 * completed reference does not reopen it.
 */
export function findMentionTrigger(
  text: string,
  caret: number,
): MentionTrigger | undefined {
  return (
    findQueryTrigger(text, caret) ??
    findIdentifierTrigger(text, caret) ??
    findBareTrigger(text, caret)
  );
}

/**
 * Recognise an identifier pasted without its scheme, which is what copying an
 * id out of the UI or another client gives you: a bare node or repo id, or a
 * full git oid naming an issue, patch or commit.
 *
 * Only the run of alphanumerics ending at the caret is considered, so
 * surrounding punctuation does not have to be stripped by the caller.
 */
function findBareTrigger(
  text: string,
  caret: number,
): MentionTrigger | undefined {
  let start = caret;
  while (start > 0 && /[0-9A-Za-z]/.test(text[start - 1])) {
    start--;
  }
  const token = text.slice(start, caret);
  if (token === "") return undefined;

  if (bareOidPattern.test(token)) {
    return { kind: "oid", start, end: caret, oid: token };
  }

  const target = parseBareIdentifier(token);

  return target ? { kind: "identifier", start, end: caret, target } : undefined;
}

function findQueryTrigger(
  text: string,
  caret: number,
): MentionTrigger | undefined {
  // Scan back from the caret for the nearest trigger, stopping at the first
  // character a query cannot contain.
  for (let index = caret - 1; index >= 0; index--) {
    const character = text[index];

    if (character === "@" || character === "#") {
      const preceding = index === 0 ? " " : text[index - 1];
      if (!openingCharacters.has(preceding)) return undefined;

      const query = text.slice(index + 1, caret);
      if (!queryPattern.test(query)) return undefined;

      // Both triggers open on the bare character, so a glance at what is
      // available costs no typing.
      //
      // A markdown heading is not a problem despite starting with `#`: the
      // space in `# Heading` is rejected by `queryPattern` above, which closes
      // the dropdown before any Enter can reach it. Only `#` followed
      // immediately by Enter — an empty heading — would be caught, and that is
      // not something people write.
      const scope: MentionScope = character === "@" ? "all" : "cobs";

      return { kind: "query", scope, start: index, end: caret, query };
    }

    if (!queryPattern.test(character)) return undefined;
    if (caret - index > maximumQueryLength) return undefined;
  }

  return undefined;
}

/**
 * Recognise a complete identifier ending exactly at the caret, so pasting a
 * DID, RID, COB reference or explorer URL offers to turn it into a link that
 * carries a readable label.
 *
 * Requiring the identifier to end at the caret, rather than merely contain
 * it, is what stops the dropdown reopening on the identifier inside a link
 * that was already inserted — that text ends in `)`.
 *
 * Nothing is offered until the identifier parses, and a DID only parses once
 * every character is present, so the dropdown appears exactly when a paste
 * completes rather than nagging through a manual keystroke at a time.
 */
function findIdentifierTrigger(
  text: string,
  caret: number,
): MentionTrigger | undefined {
  // An identifier never contains whitespace, so the word the caret sits at the
  // end of bounds the search.
  let wordStart = caret;
  while (
    wordStart > 0 &&
    !/\s/.test(text[wordStart - 1]) &&
    caret - wordStart < maximumIdentifierLength
  ) {
    wordStart--;
  }
  const word = text.slice(wordStart, caret);
  if (word === "") return undefined;

  // An identifier can be preceded by punctuation with no space, as in
  // "(did:key:z…", so try each prefix and prefer the earliest that parses.
  let best: { offset: number; target: MentionTarget } | undefined;
  for (const prefix of identifierPrefixes) {
    const offset = word.indexOf(prefix);
    if (offset === -1) continue;

    const target = parseMentionHref(word.slice(offset));
    if (!target) continue;
    if (best === undefined || offset < best.offset) {
      best = { offset, target };
    }
  }
  if (!best) return undefined;

  return {
    kind: "identifier",
    start: wordStart + best.offset,
    end: caret,
    target: best.target,
  };
}
