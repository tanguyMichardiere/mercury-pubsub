import { z } from "zod";

const StringDate = z.preprocess(
  (arg) =>
    typeof arg === "string" || typeof arg === "number" || arg instanceof Date ? new Date(arg) : arg,
  z.date()
);

export const User = z.object({
  id: z.string().uuid(),
  name: z.string(),
});
export type User = z.infer<typeof User>;

export const Session = z.object({
  accessToken: z.string().nullable(),
  expires: StringDate,
  user: User,
});
export type Session = z.infer<typeof Session>;
