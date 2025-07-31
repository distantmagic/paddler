import stylelint from "stylelint";
import config from "stylelint-config-recommended";

import { basic } from "jarmuz/job-types";

basic(async function ({ resetConsole }) {
  await resetConsole();

  const result = await stylelint.lint({
    config: {
      extends: config,
      rules: {
        "property-no-unknown": [
          true,
          {
            ignoreProperties: ["composes", "compose-with"],
          },
        ],
        "selector-pseudo-class-no-unknown": [
          true,
          {
            ignorePseudoClasses: ["global", "local"],
          },
        ],
      },
    },
    files: "resources/**/*.css",
    formatter: "string",
  });

  console.log(result.report);

  return !result.errored;
});
