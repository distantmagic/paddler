import { deepStrictEqual, strictEqual } from "node:assert/strict";
import { test } from "node:test";

import { ParsedToolCallSchema } from "../../../src/schemas/ParsedToolCall";

test("ParsedToolCallSchema keeps the inner JSON of ValidJson arguments intact", function () {
  const parsed = ParsedToolCallSchema.parse({
    id: "call_0",
    name: "get_weather",
    arguments: { ValidJson: { location: "Paris" } },
  });

  strictEqual(parsed.id, "call_0");
  strictEqual(parsed.name, "get_weather");
  deepStrictEqual(parsed.arguments, { ValidJson: { location: "Paris" } });
});
