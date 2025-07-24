import { extractHuggingFaceUrlParts } from "./extractHuggingFaceUrlParts";
import { type AgentDesiredState } from "./schemas/AgentDesiredState";

export function urlToAgentDesiredState(url: URL): AgentDesiredState {
  if (url.hostname === "huggingface.co") {
    return {
      model: {
        HuggingFace: extractHuggingFaceUrlParts(url),
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
