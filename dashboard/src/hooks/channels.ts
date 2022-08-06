import type { UseMutationResult, UseQueryResult } from "@tanstack/react-query";
import { useMutation, useQuery } from "@tanstack/react-query";
import { z } from "zod";

import { Channel } from "../types/api";
import { JSONValue } from "../types/json";
import { throwOnErrorCode, useSessionStore } from "./auth";

export function useListChannels(): UseQueryResult<Channel[]> {
  const session = useSessionStore((state) => state.session);

  return useQuery(["/api/channels"], () =>
    fetch("/api/channels", {
      headers: {
        ...(typeof session?.accessToken === "string"
          ? { Authorization: `Bearer ${session.accessToken}` }
          : {}),
        "Content-Type": "application/json",
      },
    })
      .then(throwOnErrorCode)
      .then((response) => response.json())
      .then((data) => z.array(Channel).parse(data))
  );
}

export const ChannelOptions = z.object({
  name: z.string().min(4).max(16),
  schema: JSONValue,
});
export type ChannelOptions = z.infer<typeof ChannelOptions>;

export function useCreateChannel(): UseMutationResult<Channel, unknown, ChannelOptions> {
  const session = useSessionStore((state) => state.session);

  return useMutation(["/api/channels"], ({ name, schema }) =>
    fetch("/api/channels", {
      method: "POST",
      headers: {
        ...(typeof session?.accessToken === "string"
          ? { Authorization: `Bearer ${session.accessToken}` }
          : {}),
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ name, schema }),
    })
      .then(throwOnErrorCode)
      .then((response) => response.json())
      .then((data) => Channel.parse(data))
  );
}
