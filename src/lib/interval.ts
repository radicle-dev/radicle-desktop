const dynamicIntervals = new Map<string, ReturnType<typeof setTimeout>>();

export function dynamicInterval(
  key: string,
  callback: () => void,
  period: number,
) {
  // Clear an existing interval for this key, if any.
  if (dynamicIntervals.has(key)) {
    clearTimeout(dynamicIntervals.get(key));
  }

  // Set up a new dynamic interval.
  const id = setTimeout(() => {
    callback();
    dynamicInterval(key, callback, period);
  }, period);

  dynamicIntervals.set(key, id);
}

export function resetDynamicInterval(key: string) {
  dynamicIntervals.delete(key);
}

// Run `task` now and again `period` ms after each run settles, so a slow call
// never overlaps the next one. `task` receives a check that turns false once
// the returned stop function is called, to drop a late result.
export function poll(
  task: (active: () => boolean) => Promise<void>,
  period: number,
): () => void {
  let stopped = false;
  let timer: ReturnType<typeof setTimeout> | undefined;
  const active = () => !stopped;

  const run = async () => {
    await task(active);
    if (!stopped) {
      timer = setTimeout(() => void run(), period);
    }
  };
  void run();

  return () => {
    stopped = true;
    clearTimeout(timer);
  };
}
