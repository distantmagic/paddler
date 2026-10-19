import { deepStrictEqual } from "node:assert/strict";
import { test } from "node:test";

import { ParsedToolCallSchema } from "../../../src/schemas/ParsedToolCall";

test("ParsedToolCallSchema preserves the raw string of InvalidJson arguments", function () {
  const parsed = ParsedToolCallSchema.parse({
    id: "call_1",
    name: "get_weather",
    arguments: { InvalidJson: "not json" },
  });

  deepStrictEqual(parsed.arguments, { InvalidJson: "not json" });
});
