import Head from "next/head";

import { useQuery } from "@tanstack/react-query";

import ThemeMenu from "../components/ThemeMenu";

function useHelloQuery() {
  return useQuery<string>(["/api/hello"], () =>
    fetch("/api/hello").then((response) => response.text())
  );
}

export default function Page(): JSX.Element {
  const { data } = useHelloQuery();

  return (
    <>
      <Head>
        <title>Mercury - Dashboard</title>
      </Head>
      <div className="fixed top-4 right-4">
        <ThemeMenu />
      </div>
      <div className="flex h-full flex-col items-center justify-center gap-4 p-4">
        <h1 className="text-5xl font-bold">Dashboard</h1>
        <div>{data}</div>
      </div>
    </>
  );
}
