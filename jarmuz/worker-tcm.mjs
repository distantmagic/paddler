import { join } from "node:path";
import { run } from "typed-css-modules";

import { basic } from "jarmuz/job-types";

basic(async function ({ baseDirectory }) {
  let outDir = join(baseDirectory, "resources/ts");

  await run(outDir, {
    pattern: "**/*.module.css",
    outDir,
    watch: false,
    camelCase: false,
    namedExports: false,
    dropExtension: false,
    allowArbitraryExtensions: false,
    silent: false,
    listDifferent: false,
  });
});
