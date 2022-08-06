import { z } from "zod";

import { JSONValue } from "./json";

const StringDate = z.preprocess(
  (arg) =>
    typeof arg === "string" || typeof arg === "number" || arg instanceof Date ? new Date(arg) : arg,
  z.date()
);

export const User = z.object({
  id: z.string().uuid(),
  name: z.string().min(4).max(16),
});
export type User = z.infer<typeof User>;

export const Session = z.object({
  accessToken: z.string().length(64).nullable(),
  expires: StringDate,
  user: User,
});
export type Session = z.infer<typeof Session>;

export const Channel = z.object({
  id: z.string().uuid(),
  name: z.string().min(4).max(16),
  schema: JSONValue,
});
export type Channel = z.infer<typeof Channel>;
