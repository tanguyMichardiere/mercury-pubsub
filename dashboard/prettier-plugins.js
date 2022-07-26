/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */

// @ts-expect-error no declaration
const tailwindPlugin = require("prettier-plugin-tailwindcss");
// @ts-expect-error no declaration
const sortImportsPlugin = require("@trivago/prettier-plugin-sort-imports");

module.exports = {
  parsers: {
    typescript: {
      ...tailwindPlugin.parsers.typescript,
      preprocess: sortImportsPlugin.parsers.typescript.preprocess,
    },
  },
  options: {
    ...sortImportsPlugin.options,
  },
};
