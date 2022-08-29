declare global {
  declare namespace NodeJS {
    interface ProcessEnv {
      readonly NEXT_PUBLIC_MERCURY_URL: string;
      readonly NEXT_PUBLIC_MERCURY_SUBSCRIBER_KEY: string;
      readonly MERCURY_PUBLISHER_KEY: string;
    }
  }
}

export {};
