import type { AppProps } from "next/app";

import "@fontsource/inter";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";
import { ThemeProvider } from "next-themes";

import ErrorBoundary from "../components/ErrorBoundary";

import "../styles.css";

const queryClient = new QueryClient();

export default function App({ Component, pageProps }: AppProps): JSX.Element {
  return (
    <>
      <meta content="initial-scale=1, width=device-width" name="viewport" />
      <ThemeProvider attribute="class" disableTransitionOnChange>
        <ErrorBoundary>
          <QueryClientProvider client={queryClient}>
            <Component {...pageProps} />
            <ReactQueryDevtools />
          </QueryClientProvider>
        </ErrorBoundary>
      </ThemeProvider>
    </>
  );
}
