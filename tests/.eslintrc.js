/** @type {import('eslint').Linter.Config} */
module.exports = {
  parser: "@typescript-eslint/parser",
  parserOptions: {
    tsconfigRootDir: __dirname,
    project: ["./tsconfig.json"],
  },
  plugins: ["@typescript-eslint"],
  extends: [
    "eslint:recommended",
    "plugin:@typescript-eslint/recommended",
    "plugin:@typescript-eslint/recommended-requiring-type-checking",
    "prettier",
  ],
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
