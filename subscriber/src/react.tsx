import type { ReactNode } from "react";
import { createContext, useContext, useEffect, useMemo } from "react";

import type { Channels } from "@mercury-pubsub/types";
import useEvent from "react-use-event-hook";

import type { SubscribeOptions } from ".";
import Subscriber from ".";

const subscriberContext = createContext<Subscriber | undefined>(undefined);

export default function SubscriberProvider(props: {
  url: string;
  _key: string;
  children: ReactNode;
}): JSX.Element {
  const subscriber = useMemo(() => new Subscriber(props.url, props._key), [props.url, props._key]);

  return (
    <subscriberContext.Provider value={subscriber}>{props.children}</subscriberContext.Provider>
  );
}

export function useSubscribe<C extends keyof Channels>(
  channel: C,
  events: Omit<SubscribeOptions<C>, "signal">
): void {
  const subscriber = useContext(subscriberContext);

  if (subscriber === undefined) {
    throw new Error("no subscriber context");
  }

  const abortController = useMemo(() => new AbortController(), []);

  // eslint-disable-next-line @typescript-eslint/no-empty-function
  const onopen = useEvent(events.onopen ?? async function () {});
  const ondata = useEvent(events.ondata);
  // eslint-disable-next-line @typescript-eslint/no-empty-function
  const onclose = useEvent(events.onclose ?? function () {});

  useEffect(
    function () {
      subscriber
        .subscribe(channel, { signal: abortController.signal, onopen, ondata, onclose })
        .catch(function (error) {
          console.error(error);
        });
      return function () {
        abortController.abort();
      };
    },
    [subscriber, channel, abortController, onopen, ondata, onclose]
  );
}
