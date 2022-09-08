module.exports = {
  extends: ["plugin:astro/recommended", "plugin:astro/jsx-a11y-recommended"],
  // TODO: config for js config files and zx scripts
  overrides: [
    {
      files: ["*.astro"],
      parser: "astro-eslint-parser",
      parserOptions: {
        parser: "@typescript-eslint/parser",
        extraFileExtensions: [".astro"],
      },
      rules: {},
    },
  ],
};
