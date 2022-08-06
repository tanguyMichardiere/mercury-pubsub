import { z } from "zod";

export const JSONPrimitive = z.union([z.string(), z.number(), z.boolean(), z.null()]);
export type JSONPrimitive = z.infer<typeof JSONPrimitive>;

export const JSONArray = z.lazy(() => z.array(JSONValue));
export type JSONArray = z.infer<typeof JSONArray>;

export const JSONObject = z.lazy(() => z.record(JSONValue));
export type JSONObject = z.infer<typeof JSONObject>;

export type JSONValue = { [key: string]: JSONValue } | JSONValue[] | JSONPrimitive;
export const JSONValue: z.ZodType<JSONValue> = z.lazy(() =>
  z.union([z.record(JSONValue), z.array(JSONValue), JSONPrimitive])
);
