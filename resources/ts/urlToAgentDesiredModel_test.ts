import test from "ava";
import { urlToAgentDesiredModel } from "./urlToAgentDesiredModel";

test("recognizes Hugging Face urls", function (test) {
  const url = new URL(
    "https://huggingface.co/Qwen/Qwen3-0.6B-GGUF/blob/main/Qwen3-0.6B-Q8_0.gguf",
  );

  test.deepEqual(urlToAgentDesiredModel(url), {
    HuggingFace: {
      filename: "Qwen3-0.6B-Q8_0.gguf",
      repo_id: "Qwen/Qwen3-0.6B-GGUF",
      revision: "main",
    },
  });
});

test("uses local urls", function (test) {
  const url = new URL("file:///home/user/models/Qwen3-0.6B-Q8_0.gguf");

  test.deepEqual(urlToAgentDesiredModel(url), {
    Local: "/home/user/models/Qwen3-0.6B-Q8_0.gguf",
  });
});
