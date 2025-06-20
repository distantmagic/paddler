#!/usr/bin/env node

import { run } from "./jarmuz/run-website.mjs";

run({
  development: false,
  once: true,
  rustJobs: ["cargo-build-release"],
});
