import { useEffect } from "react";

import Head from "next/head";
import { useRouter } from "next/router";

import { useFloating } from "@floating-ui/react-dom";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { z } from "zod";

import { useIsFirstSignin, useLogin, useSession } from "../hooks/auth";

const Schema = z.object({
  name: z.string().min(4),
  password: z.string().min(8),
});
type Schema = z.infer<typeof Schema>;

export default function Page(): JSX.Element {
  const router = useRouter();
  const session = useSession();
  const isFirstSignin = useIsFirstSignin();

  useEffect(
    function () {
      if (session !== undefined && session !== null) {
        void router.push("/");
      }
    },
    [session, router]
  );

  useEffect(
    function () {
      if (isFirstSignin.data === true) {
        void router.push("/create-user");
      }
    },
    [isFirstSignin.data, router]
  );

  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<Schema>({
    resolver: zodResolver(Schema),
  });

  const login = useLogin();

  const nameErrorFloating = useFloating({ placement: "right" });
  const passwordErrorFloating = useFloating({ placement: "right" });

  return (
    <>
      <Head>
        <title>Mercury - Login</title>
      </Head>
      <div className="flex h-full flex-col items-center justify-center gap-4">
        <h1>Login</h1>
        <form
          className="flex flex-col gap-4 rounded-md p-4 shadow-md dark:bg-gray-800"
          onSubmit={handleSubmit((data) => login.mutate(data))}
        >
          <div ref={nameErrorFloating.reference}>
            <input
              className="block w-full rounded-md border-gray-300 focus:border-indigo-500 focus:ring-indigo-500 dark:bg-gray-800 sm:text-sm"
              placeholder="name"
              type="text"
              {...register("name")}
            />
          </div>
          {errors.name !== undefined && (
            <p
              className="ml-1 rounded-md bg-white p-1 text-red-500 shadow-md dark:bg-gray-700 dark:text-red-300"
              ref={nameErrorFloating.floating}
              style={{
                position: nameErrorFloating.strategy,
                top: nameErrorFloating.y ?? 0,
                left: nameErrorFloating.x ?? 0,
              }}
            >
              {errors.name.message}
            </p>
          )}
          <div ref={passwordErrorFloating.reference}>
            <input
              className="block w-full rounded-md border-gray-300 focus:border-indigo-500 focus:ring-indigo-500 dark:bg-gray-800 sm:text-sm"
              placeholder="password"
              type="password"
              {...register("password")}
            />
          </div>
          {errors.password !== undefined && (
            <p
              className="ml-1 rounded-md bg-white p-1 text-red-500 shadow-md dark:bg-gray-700 dark:text-red-300"
              ref={passwordErrorFloating.floating}
              style={{
                position: passwordErrorFloating.strategy,
                top: passwordErrorFloating.y ?? 0,
                left: passwordErrorFloating.x ?? 0,
              }}
            >
              {errors.password.message}
            </p>
          )}
          <button
            className="inline-flex justify-center rounded-md border border-transparent bg-indigo-600 py-2 px-4 text-sm font-medium text-white shadow-sm hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2"
            disabled={login.status === "loading"}
            type="submit"
          >
            Log in
          </button>
        </form>
      </div>
    </>
  );
}
