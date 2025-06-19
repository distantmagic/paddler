import { jarmuz } from "jarmuz";

export function run({ development, once = false, rustJobs }) {
  const esbuildJob = development ? "esbuild-development" : "esbuild-production";

  jarmuz({
    once,
    pipeline: ["stylelint", "tcm", "eslint", "tsc", esbuildJob, ...rustJobs],
    watch: ["resources", "src", "templates"],
  }).decide(function ({ matches, schedule }) {
    if (matches("resources/**/*.css")) {
      schedule("stylelint");
    }

    switch (true) {
      case matches("resources/ts/**/*.module.css"):
        schedule("tcm");
        return;
      case matches("resources/**/*.{ts,tsx}"):
        schedule("eslint");
        schedule("tsc");
        break;
      case matches("resources/css/**/*.css"):
        schedule(esbuildJob);
        return;
      case matches("src/**/*.rs"):
        for (const job of rustJobs) {
          schedule(job);
        }
        return;
    }
  });
}
