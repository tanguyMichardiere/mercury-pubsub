import { createContext, useContext, useEffect, useRef } from "react";

import type { Channels, SubscribeOptions } from ".";
import Subscriber from ".";

const subscriberContext = createContext<Subscriber | undefined>(undefined);

export default function SubscriberProvider(props: { url: string; key: string }): JSX.Element {
  const subscriber = useRef(new Subscriber(props.url, props.key));

  return <subscriberContext.Provider value={subscriber.current}></subscriberContext.Provider>;
}

export function useSubscribe<C extends keyof Channels>(
  channel: C,
  { onopen, ondata, onclose }: SubscribeOptions<C>
): void {
  const subscriber = useContext(subscriberContext);

  if (subscriber === undefined) {
    throw new Error("no subscriber context");
  }

  const abortController = useRef(new AbortController()).current;

  useEffect(function () {
    subscriber
      .subscribe(channel, { signal: abortController.signal, onopen, ondata, onclose })
      .catch(function (error) {
        console.error(error);
      });
    return function () {
      abortController.abort();
    };
  });
}
