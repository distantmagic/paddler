import { command } from "jarmuz/job-types";

command(`
  npm exec prettier --
    --plugin=prettier-plugin-organize-imports
    --write
    integration_tests/tests/fixtures
    jarmuz
    resources
    *.mjs
`);
