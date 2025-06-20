#!/usr/bin/env node

import { run } from "./jarmuz/run-website.mjs";

run({
  development: true,
  once: false,
  rustJobs: ["cargo-build", "paddler"],
});
