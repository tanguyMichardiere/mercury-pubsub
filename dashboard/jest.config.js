/** @type {import('next/jest').default} */
// HACK
// @ts-expect-error we have to use nextJest as nextJest.default
const nextJest = require("next/jest");

const createJestConfig = nextJest({
  dir: "./",
});

module.exports = createJestConfig({
  setupFilesAfterEnv: ["<rootDir>/jest.setup.js"],
  moduleDirectories: ["node_modules", "<rootDir>/"],
  testEnvironment: "jest-environment-jsdom",
  testMatch: ["<rootDir>/tests/**/*.test.{ts,tsx}"],
  collectCoverage: true,
  collectCoverageFrom: ["src/components/*.{ts,tsx}", "src/hooks/*.{ts,tsx}"],
  coverageThreshold: {
    global: {
      branches: 80,
      functions: 80,
      lines: 80,
      statements: 80,
    },
  },
});
