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
