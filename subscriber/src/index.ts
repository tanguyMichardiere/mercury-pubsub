import type { Channels } from "@mercury-pubsub/types";
import type { EventSourceMessage, FetchEventSourceInit } from "@microsoft/fetch-event-source";
import { fetchEventSource } from "@microsoft/fetch-event-source";

export type SubscribeOptions<C extends keyof Channels> = Pick<
  FetchEventSourceInit,
  "signal" | "onopen" | "onclose"
> & { ondata: (data: Channels[C]) => void };

export default class Subscriber {
  #url: string;
  #key: string;

  constructor(url: string, key: string) {
    this.#url = url;
    this.#key = key;
  }

  async subscribe<C extends keyof Channels>(
    channel: C,
    { signal, onopen, ondata, onclose }: SubscribeOptions<C>
  ): Promise<void> {
    const url = new URL(`/sse/${channel}`, this.#url);
    function onmessage(ev: EventSourceMessage) {
      ondata(JSON.parse(ev.data));
    }
    await fetchEventSource(url.href, {
      signal,
      headers: { Authorization: `Bearer ${this.#key}` },
      onopen,
      onmessage,
      onclose,
      openWhenHidden: true,
    });
  }
}
