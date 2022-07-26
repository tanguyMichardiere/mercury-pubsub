/** @type {import('eslint').Linter.Config} */
module.exports = {
  parser: "@typescript-eslint/parser",
  parserOptions: {
    tsconfigRootDir: __dirname,
    project: ["./tsconfig.json"],
  },
  plugins: ["@typescript-eslint", "tsdoc"],
  extends: [
    "eslint:recommended",
    "plugin:@typescript-eslint/recommended",
    "plugin:@typescript-eslint/recommended-requiring-type-checking",
    "plugin:react/recommended",
    "plugin:react/jsx-runtime",
    "plugin:react-hooks/recommended",
    "prettier",
  ],
  settings: {
    react: {
      version: "detect",
    },
  },
  overrides: [
    {
      files: ["./*.js"],
      rules: {
        "@typescript-eslint/no-var-requires": "off",
        "tsdoc/syntax": "off",
      },
    },
  ],
  rules: {
    "arrow-body-style": ["warn", "never"],
    camelcase: "warn",
    "func-style": [
      "warn",
      "declaration",
      {
        allowArrowFunctions: true,
      },
    ],
    "no-alert": "warn",
    "no-else-return": "warn",
    "no-implicit-coercion": "warn",
    "no-return-assign": "warn",
    "no-throw-literal": "warn",
    "no-useless-return": "warn",
    "no-void": ["warn", { allowAsStatement: true }],
    "react/hook-use-state": "warn",
    "react/jsx-boolean-value": ["warn", "never"],
    "react/jsx-curly-brace-presence": [
      "warn",
      { props: "never", children: "never", propElementValues: "always" },
    ],
    "react/jsx-fragments": ["warn", "syntax"],
    "react/jsx-no-constructed-context-values": "warn",
    "react/jsx-no-script-url": "warn",
    "react/jsx-no-useless-fragment": ["warn", { allowExpressions: true }],
    "react/jsx-sort-props": "warn",
    "react/no-array-index-key": "warn",
    "react/no-danger": "warn",
    "react/no-invalid-html-attribute": "warn",
    "react/no-multi-comp": "warn",
    "react/no-namespace": "warn",
    "react/no-unstable-nested-components": "warn",
    "react/self-closing-comp": "warn",
    "tsdoc/syntax": "warn",
    "@typescript-eslint/consistent-type-assertions": ["warn", { assertionStyle: "never" }],
    "@typescript-eslint/consistent-type-definitions": ["warn", "type"],
    "@typescript-eslint/consistent-type-exports": [
      "warn",
      { fixMixedExportsWithInlineTypeSpecifier: false },
    ],
    "@typescript-eslint/consistent-type-imports": [
      "warn",
      { prefer: "type-imports", disallowTypeAnnotations: true },
    ],
    "@typescript-eslint/explicit-module-boundary-types": ["warn"],
    "@typescript-eslint/no-floating-promises": ["warn", { ignoreVoid: true }],
    "@typescript-eslint/no-misused-promises": [
      "warn",
      {
        checksVoidReturn: {
          arguments: false,
          attributes: false,
        },
      },
    ],
    "@typescript-eslint/no-unnecessary-type-constraint": "warn",
    "@typescript-eslint/no-unused-vars": "off",
    "@typescript-eslint/non-nullable-type-assertion-style": "warn",
    "@typescript-eslint/prefer-for-of": "warn",
    "@typescript-eslint/prefer-nullish-coalescing": "warn",
    "@typescript-eslint/strict-boolean-expressions": [
      "warn",
      {
        allowString: false,
        allowNumber: false,
        allowNullableObject: false,
      },
    ],
    "@typescript-eslint/switch-exhaustiveness-check": "warn",
  },
};
