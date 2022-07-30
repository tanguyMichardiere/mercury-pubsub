import Head from "next/head";
import { useRouter } from "next/router";

import Spinner from "../components/Spinner";
import ThemeMenu from "../components/ThemeMenu";

import { useLogOut, useSession } from "../hooks/auth";

export default function Page(): JSX.Element {
  const router = useRouter();
  const session = useSession();
  const logOut = useLogOut();

  if (session === undefined) {
    return (
      <div className="flex h-full items-center justify-center">
        <Spinner />
      </div>
    );
  }

  if (session === null) {
    void router.push("/login");
    return (
      <div className="flex h-full items-center justify-center">
        <Spinner />
      </div>
    );
  }

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
        <div>Authenticated as: {session.user.name}</div>
        <button
          className="inline-flex justify-center rounded-md border border-transparent bg-indigo-600 py-2 px-4 text-sm font-medium text-white shadow-sm hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2"
          onClick={logOut}
          type="button"
        >
          Log out
        </button>
      </div>
    </>
  );
}
