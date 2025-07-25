import { extractHuggingFaceUrlParts } from "./extractHuggingFaceUrlParts";
import { type AgentDesiredModel } from "./schemas/AgentDesiredModel";

export function urlToAgentDesiredModel(url: URL): AgentDesiredModel {
  if (url.hostname === "huggingface.co") {
    return {
      HuggingFace: extractHuggingFaceUrlParts(url),
    };
  } else if (url.protocol === "file:") {
    return {
      Local: url.pathname,
    };
  } else {
    throw new Error("Unsupported URL format");
  }
}
