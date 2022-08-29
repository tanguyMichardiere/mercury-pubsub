import type { AppProps } from "next/app";

import SubscriberProvider from "@mercury-pubsub/subscriber/react";

import "../styles/globals.css";

function MyApp({ Component, pageProps }: AppProps) {
  return (
    <SubscriberProvider
      url={process.env.NEXT_PUBLIC_MERCURY_URL}
      _key={process.env.NEXT_PUBLIC_MERCURY_SUBSCRIBER_KEY}
    >
      <Component {...pageProps} />
    </SubscriberProvider>
  );
}

export default MyApp;
