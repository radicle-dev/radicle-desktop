import { z, type SafeParseReturnType } from "zod";

export default function useLocalStorage<
  S extends z.infer<T>,
  T extends z.ZodType = z.ZodType<S>,
>(
  key: string,
  schema: T,
  initialValue: z.infer<typeof schema>,
  disableLocalStorage = false,
) {
  const stored = !disableLocalStorage ? localStorage.getItem(key) : null;

  const parseFromJson = (
    content: string,
  ): SafeParseReturnType<string, T["_output"]> => {
    return z
      .string()
      .transform((_, ctx) => {
        try {
          return JSON.parse(content);
        } catch {
          ctx.addIssue({
            code: z.ZodIssueCode.custom,
            message: "invalid json",
          });
          return z.never;
        }
      })
      .pipe(schema)
      .safeParse(content);
  };

  let value = $state<S>(initialValue);

  if (stored) {
    try {
      const parsed = parseFromJson(stored);
      if (parsed.success) {
        value = parsed.data;
      } else {
        console.error("Invalid stored data:", parsed.error);
      }
    } catch (error) {
      console.error("Error parsing stored data:", error);
    }
  }

  return {
    get value() {
      return value;
    },
    set value(v: S) {
      value = v;
      if (!disableLocalStorage)
        localStorage.setItem(key, JSON.stringify(value));
    },
    clear() {
      value = initialValue;
      if (!disableLocalStorage) localStorage.removeItem(key);
    },
  };
}
