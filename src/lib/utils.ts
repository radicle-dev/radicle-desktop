import bs58 from "bs58";

export const unreachable = (value: never): never => {
  throw new Error(`Unreachable code: ${value}`);
};

export function formatRepositoryId(id: string): string {
  const parsedId = parseRepositoryId(id);

  if (parsedId) {
    return `${parsedId.prefix}${truncateId(parsedId.pubkey)}`;
  }

  return id;
}

export function parseRepositoryId(
  rid: string,
): { prefix: string; pubkey: string } | undefined {
  const match = /^(rad:)?(z[a-zA-Z0-9]+)$/.exec(rid);
  if (match) {
    const hex = bs58.decode(match[2].substring(1));
    if (hex.byteLength !== 20) {
      return undefined;
    }

    return { prefix: match[1] || "rad:", pubkey: match[2] };
  }

  return undefined;
}

export function truncateId(pubkey: string): string {
  return `${pubkey.substring(0, 6)}…${pubkey.slice(-6)}`;
}

export function formatOid(id: string): string {
  return id.substring(0, 7);
}

export const formatTimestamp = (
  timestamp: number,
  current = new Date().getTime(),
): string => {
  const units: Record<string, number> = {
    year: 24 * 60 * 60 * 1000 * 365,
    month: (24 * 60 * 60 * 1000 * 365) / 12,
    day: 24 * 60 * 60 * 1000,
    hour: 60 * 60 * 1000,
    minute: 60 * 1000,
    second: 1000,
  };

  // Multiplying timestamp with 1000 to convert from seconds to milliseconds
  timestamp = timestamp * 1000;
  const rtf = new Intl.RelativeTimeFormat("en", {
    numeric: "auto",
    style: "long",
  });
  const elapsed = current - timestamp;

  if (elapsed > units["year"]) {
    return "more than a year ago";
  } else if (elapsed < 0) {
    return "now"; // If elapsed is a negative number we are dealing with an item from the future, and we return "now"
  }

  for (const u in units) {
    if (elapsed > units[u] || u === "second") {
      // We convert the division result to a negative number to get "XX [unit] ago"
      return rtf.format(
        Math.round(elapsed / units[u]) * -1,
        u as Intl.RelativeTimeFormatUnit,
      );
    }
  }

  return new Date(timestamp).toUTCString();
};
