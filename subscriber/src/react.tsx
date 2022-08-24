import { createContext, useRef } from "react";

import type { Channels, SubscribeOptions } from ".";
import Subscriber from ".";

const subscriberContext = createContext<Subscriber>(undefined as unknown as Subscriber);

export default function SubscriberProvider(props: { url: string; key: string }): JSX.Element {
  const subscriber = useRef(new Subscriber(props.url, props.key));

  return <subscriberContext.Provider value={subscriber.current}></subscriberContext.Provider>;
}

export function useSubscribe<C extends Channels>(
  channel: C,
  { onopen, ondata, onclose }: SubscribeOptions<C>
) {}
