import { useEffect } from "react";

import type { UseMutationResult, UseQueryResult } from "@tanstack/react-query";
import { useMutation, useQuery } from "@tanstack/react-query";
import { z } from "zod";
import create from "zustand";

import { Session } from "../types/api";

type SessionStore = {
  session: Session | null | undefined;
  setSession: (session: Session | null) => void;
  refreshSession: () => Promise<void>;
  logOut: () => Promise<void>;
};

export const useSessionStore = create<SessionStore>((set) => ({
  session: undefined,
  setSession(session) {
    set({ session });
  },
  async refreshSession() {
    const response = await fetch("/api/auth/refresh", { method: "POST" });
    if (response.ok) {
      const session = Session.parse(await response.json());
      set({ session });
    } else {
      set({ session: null });
    }
  },
  async logOut() {
    await fetch("/api/auth/logout", { method: "POST" });
    set({ session: null });
  },
}));

export function throwOnErrorCode(response: Response): Response {
  if (!response.ok) throw new Error(response.status.toString());
  return response;
}

export function useIsFirstSignin(): UseQueryResult<boolean> {
  return useQuery(["/api/auth/user-count"], () =>
    fetch("/api/auth/user-count")
      .then(throwOnErrorCode)
      .then((response) => response.json())
      .then((data) => z.number().int().parse(data))
      .then((count) => count === 0)
  );
}

export function useSession(): Session | null | undefined {
  const session = useSessionStore((state) => state.session);
  const refreshSession = useSessionStore((state) => state.refreshSession);

  useEffect(
    function () {
      if (session === undefined || session === null || session.accessToken === null) {
        void refreshSession();
      }
    },
    [session, refreshSession]
  );

  return session;
}

export const LoginOptions = z.object({
  name: z.string().min(4).max(16),
  password: z.string().min(8),
});
export type LoginOptions = z.infer<typeof LoginOptions>;

export function useLogin(): UseMutationResult<Session, unknown, LoginOptions> {
  const setSession = useSessionStore((state) => state.setSession);

  return useMutation(
    ["/api/auth/login"],
    ({ name, password }) =>
      fetch("/api/auth/login", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ name, password }),
      })
        .then(throwOnErrorCode)
        .then((response) => response.json().then((data) => Session.parse(data))),
    {
      onSuccess(data) {
        setSession(data);
      },
    }
  );
}

export function useLogOut(): () => Promise<void> {
  return useSessionStore((state) => state.logOut);
}

export const CreateUserOptions = z.object({
  name: z.string().min(4).max(16),
  password: z.string().min(8),
});
export type CreateUserOptions = z.infer<typeof CreateUserOptions>;

export function useCreateUser(): UseMutationResult<Session | null, unknown, CreateUserOptions> {
  const { session, setSession } = useSessionStore();

  return useMutation(
    ["/api/auth/create-user"],
    ({ name, password }) =>
      fetch("/api/auth/create-user", {
        method: "POST",
        headers: {
          ...(typeof session?.accessToken === "string"
            ? { Authorization: `Bearer ${session.accessToken}` }
            : {}),
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ name, password }),
      })
        .then(throwOnErrorCode)
        .then((response) =>
          response.status === 201 ? null : response.json().then((data) => Session.parse(data))
        ),
    {
      onSuccess(data) {
        if (data !== null) {
          setSession(data);
        }
      },
    }
  );
}
