import image from "@astrojs/image";
import prefetch from "@astrojs/prefetch";
import tailwind from "@astrojs/tailwind";
import { defineConfig } from "astro/config";

// https://astro.build/config
export default defineConfig({
  integrations: [
    image(),
    prefetch({
      // prefetch links with an href that begins with `/`
      selector: "a[href^='/']",
    }),
    tailwind(),
  ],
});
