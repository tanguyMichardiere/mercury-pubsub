declare global {
  declare namespace NodeJS {
    // eslint-disable-next-line @typescript-eslint/consistent-type-definitions
    interface ProcessEnv {
      readonly NEXT_PUBLIC_TITLE: string;
      readonly NEXT_PUBLIC_DESCRIPTION: string;
    }
  }
}

export {};
