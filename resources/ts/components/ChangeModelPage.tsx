import React from "react";

import { useAgentDesiredState } from "../hooks/useAgentDesiredState";
import { matchFetchJsonState } from "../matchFetchJsonState";
import { type AgentDesiredModel } from "../schemas/AgentDesiredModel";
import { ChangeModelForm } from "./ChangeModelForm";
import { FloatingStatus } from "./FloatingStatus";
import { InferenceParametersContextProvider } from "./InferenceParametersContextProvider";

function modelSchemaToUrl(model: AgentDesiredModel): string {
  if (model === "None") {
    return "";
  }

  if ("HuggingFace" in model) {
    const { HuggingFace } = model;

    return `https://huggingface.co/${HuggingFace.repo_id}/blob/${HuggingFace.revision}/${HuggingFace.filename}`;
  }

  if ("Local" in model) {
    return `file://${model.Local}`;
  }

  throw new Error(`Unsupported model schema: ${JSON.stringify(model)}`);
}

export function ChangeModelPage({
  managementAddr,
}: {
  managementAddr: string;
}) {
  const loadingState = useAgentDesiredState({ managementAddr });

  return matchFetchJsonState(loadingState, {
    empty() {
      return (
        <FloatingStatus>Unable to pick the desired state source</FloatingStatus>
      );
    },
    error({ error }) {
      return (
        <FloatingStatus>Error loading desired state: {error}</FloatingStatus>
      );
    },
    loading() {
      return <FloatingStatus>Loading desired state...</FloatingStatus>;
    },
    ok({ response: { inference_parameters, model } }) {
      return (
        <InferenceParametersContextProvider
          defaultInferenceParameters={inference_parameters}
        >
          <ChangeModelForm
            defaultModelUri={modelSchemaToUrl(model)}
            managementAddr={managementAddr}
          />
        </InferenceParametersContextProvider>
      );
    },
  });
}
