import { publicKeyFromDid } from "@app/lib/utils";

export function isDelegate(
  publicKey: string | undefined,
  delegates: string[],
): true | undefined {
  if (!publicKey) {
    return undefined;
  }
  return (
    delegates.some(delegate => publicKeyFromDid(delegate) === publicKey) ||
    undefined
  );
}

export function isDelegateOrAuthor(
  publicKey: string | undefined,
  delegates: string[],
  author: string,
): true | undefined {
  return (
    isDelegate(publicKey, delegates) ||
    publicKey === publicKeyFromDid(author) ||
    undefined
  );
}
