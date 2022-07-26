/** @type {import('prettier').Config} */
module.exports = {
  pluginSearchDirs: false,
  plugins: ["./prettier-plugins"],
  printWidth: 100,
  importOrder: [
    "^react$",
    "^next(/.*)?$",
    "<THIRD_PARTY_MODULES>",
    "^\\.[./]*/components/",
    "^\\.",
  ],
  importOrderSeparation: true,
  importOrderSortSpecifiers: true,
};
