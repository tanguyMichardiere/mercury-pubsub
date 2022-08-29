/** @type {import('prettier').Config} */
module.exports = {
  printWidth: 100,
  importOrder: ["^react$", "<THIRD_PARTY_MODULES>", "^\\."],
  importOrderSeparation: true,
  importOrderSortSpecifiers: true,
};
