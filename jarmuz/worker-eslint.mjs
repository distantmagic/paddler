import { ESLint } from "eslint";
import { glob } from "glob";

import { basic } from "jarmuz/job-types";

const eslint = new ESLint({
  cacheStrategy: "content",
});

basic(async function ({ buildId, resetConsole }) {
  await resetConsole();

  console.log(`ESLint with ID: ${buildId}`);

  const results = await eslint.lintFiles(
    await glob(["jarmuz/**/*.mjs", "resources/ts/**/*.{ts,tsx}"]),
  );

  const formatter = await eslint.loadFormatter("stylish");
  const resultText = formatter.format(results);

  const hasErrors = results.some(function (result) {
    return result.errorCount > 0 || result.warningCount > 0;
  });

  console.log(resultText);

  return !hasErrors;
});
