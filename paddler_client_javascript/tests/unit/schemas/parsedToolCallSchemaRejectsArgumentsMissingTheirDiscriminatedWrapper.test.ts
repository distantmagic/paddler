import { throws } from "node:assert/strict";
import { test } from "node:test";

import { ParsedToolCallSchema } from "../../../src/schemas/ParsedToolCall";

test("ParsedToolCallSchema rejects arguments missing their discriminated wrapper", function () {
  throws(function () {
    ParsedToolCallSchema.parse({
      id: "call_2",
      name: "get_weather",
      arguments: { location: "Paris" },
    });
  });
});
