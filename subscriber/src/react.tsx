import type { ReactNode } from "react";
import { createContext, useContext, useEffect, useRef } from "react";

import type { Channels, SubscribeOptions } from ".";
import Subscriber from ".";

const subscriberContext = createContext<Subscriber | undefined>(undefined);

export default function SubscriberProvider(props: {
  url: string;
  _key: string;
  children: ReactNode;
}): JSX.Element {
  const subscriber = useRef(new Subscriber(props.url, props._key));

  return (
    <subscriberContext.Provider value={subscriber.current}>
      {props.children}
    </subscriberContext.Provider>
  );
}

export function useSubscribe<C extends keyof Channels>(
  channel: C,
  { onopen, ondata, onclose }: SubscribeOptions<C>
): void {
  const subscriber = useContext(subscriberContext);

  if (subscriber === undefined) {
    throw new Error("no subscriber context");
  }

  const abortController = useRef(new AbortController());

  useEffect(function () {
    subscriber
      .subscribe(channel, { signal: abortController.current.signal, onopen, ondata, onclose })
      .catch(function (error) {
        console.error(error);
      });
    return function () {
      abortController.current.abort();
    };
  });
}
