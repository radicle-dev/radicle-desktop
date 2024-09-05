import { object, string, z } from "zod";

export type Author = z.infer<typeof authorSchema>;

export const authorSchema = object({
  id: string(),
  alias: string().optional(),
});
