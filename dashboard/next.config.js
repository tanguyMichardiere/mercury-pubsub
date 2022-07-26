/** @typedef {import('next').NextConfig} NextConfig */

/** @typedef {NextConfig & {pwa: any}} NextConfigWithPWA */

/**
 * @callback WithPWA
 * @param {NextConfigWithPWA} nextConfigWithPWA
 * @returns {NextConfig}
 */

/** @type {WithPWA} */
// HACK
// @ts-expect-error next-pwa doesn't provide type definitions
// eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
const withPWA = require("next-pwa");

/** @type {NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  compiler: {
    removeConsole: process.env.NODE_ENV === "production",
  },
  /** @param {import('webpack').Configuration} config */
  webpack(config) {
    return config;
  },
  // TODO: remove when no more experimental features
  experimental: {
    images: { allowFutureImage: true },
    newNextLinkBehavior: true,
  },
  eslint: {
    dirs: ["src"],
  },
  rewrites() {
    return Promise.resolve([
      {
        source: "/api/:path*",
        destination: "http://localhost:8080/api/:path*",
      },
    ]);
  },
};

const nextConfigWithPWA = withPWA({
  ...nextConfig,
  pwa: {
    dest: "public",
    disable: process.env.NODE_ENV !== "production",
  },
});

if (process.env["ANALYZE"] === "true") {
  /**
   * @callback WithBundleAnalyzer
   * @param {NextConfig} nextConfig
   * @returns {NextConfig}
   */

  /** @type {WithBundleAnalyzer} */
  // HACK
  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-call
  const withBundleAnalyzer = require("@next/bundle-analyzer")();
  module.exports = withBundleAnalyzer(nextConfigWithPWA);
} else {
  module.exports = nextConfigWithPWA;
}
