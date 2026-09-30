import { afterEach, beforeEach, expect, test, vi } from "vitest";

import { dynamicInterval, resetDynamicInterval } from "@app/lib/interval";

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  resetDynamicInterval("a");
  resetDynamicInterval("b");
  vi.useRealTimers();
});

test("runs the callback once per period", () => {
  const callback = vi.fn();
  dynamicInterval("a", callback, 100);

  vi.advanceTimersByTime(99);
  expect(callback).not.toHaveBeenCalled();
  vi.advanceTimersByTime(1);
  expect(callback).toHaveBeenCalledTimes(1);
  vi.advanceTimersByTime(200);
  expect(callback).toHaveBeenCalledTimes(3);
});

test("rescheduling a key replaces its callback and period", () => {
  const first = vi.fn();
  const second = vi.fn();
  dynamicInterval("a", first, 100);
  vi.advanceTimersByTime(50);
  dynamicInterval("a", second, 300);

  vi.advanceTimersByTime(299);
  expect(first).not.toHaveBeenCalled();
  expect(second).not.toHaveBeenCalled();
  vi.advanceTimersByTime(1);
  expect(second).toHaveBeenCalledTimes(1);
});

test("an async callback can reschedule its own key", async () => {
  const callback = vi.fn(async () => {
    await Promise.resolve();
    dynamicInterval("a", callback, 500);
  });
  dynamicInterval("a", callback, 100);

  await vi.advanceTimersByTimeAsync(100);
  expect(callback).toHaveBeenCalledTimes(1);
  await vi.advanceTimersByTimeAsync(499);
  expect(callback).toHaveBeenCalledTimes(1);
  await vi.advanceTimersByTimeAsync(1);
  expect(callback).toHaveBeenCalledTimes(2);
});

test("resetting a key stops its callback", () => {
  const callback = vi.fn();
  dynamicInterval("a", callback, 100);
  resetDynamicInterval("a");

  vi.advanceTimersByTime(1000);
  expect(callback).not.toHaveBeenCalled();
});

test("keys run independently", () => {
  const a = vi.fn();
  const b = vi.fn();
  dynamicInterval("a", a, 100);
  dynamicInterval("b", b, 100);

  vi.advanceTimersByTime(100);
  expect(a).toHaveBeenCalledTimes(1);
  expect(b).toHaveBeenCalledTimes(1);
});
