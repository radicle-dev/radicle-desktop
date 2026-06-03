import { LRUCache } from "lru-cache";

export interface Cached<Args extends unknown[], V> {
  (...args: Args): Promise<V>;
  /** Drop all cached entries, forcing the next call to re-fetch. */
  clear(): void;
}

export function cached<Args extends unknown[], V>(
  f: (...args: Args) => Promise<V>,
  makeKey: (...args: Args) => string,
  options?: LRUCache.Options<string, { promise: Promise<V> }, unknown>,
): Cached<Args, V> {
  const cache = new LRUCache(options || { max: 500 });
  const fn = function (...args: Args): Promise<V> {
    const key = makeKey(...args);
    const hit = cache.get(key);
    if (hit !== undefined) return hit.promise;

    const promise = f(...args);
    // Evict on rejection so the next caller can retry with a fresh request.
    promise.catch(() => {
      cache.delete(key);
    });
    cache.set(key, { promise });
    return promise;
  } as Cached<Args, V>;
  fn.clear = () => cache.clear();
  return fn;
}
