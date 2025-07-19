import { type AgentDesiredState } from "./schemas/AgentDesiredState";

export function urlToAgentDesiredState(url: URL): AgentDesiredState {
  if (url.hostname === "huggingface.co") {
    const parts = url.pathname.split("/");
    const filename = parts.pop();

    if (!filename) {
      throw new Error("Invalid Hugging Face URL: No filename found");
    }

    const repo = parts.slice(1, 3).join("/");

    return {
      model: {
        HuggingFace: {
          filename,
          repo,
        },
      },
    };
  } else if (url.protocol === "file:") {
    return {
      model: {
        Local: url.pathname,
      },
    };
  } else {
    throw new Error("Unsupported URL format");
  }
}
