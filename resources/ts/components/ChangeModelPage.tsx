import React from "react";

import { useBalancerDesiredState } from "../hooks/useBalancerDesiredState";
import { matchFetchJsonState } from "../matchFetchJsonState";
import { type AgentDesiredModel } from "../schemas/AgentDesiredModel";
import { ChangeModelForm } from "./ChangeModelForm";
import { ChatTemplateContextProvider } from "./ChatTemplateContextProvider";
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
  const loadingState = useBalancerDesiredState({ managementAddr });

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
    ok({
      response: {
        chat_template_override,
        inference_parameters,
        model,
        use_chat_template_override,
      },
    }) {
      return (
        <ChatTemplateContextProvider
          defaultChatTemplateOverride={chat_template_override}
          defaultUseChatTemplateOverride={use_chat_template_override}
        >
          <InferenceParametersContextProvider
            defaultInferenceParameters={inference_parameters}
          >
            <ChangeModelForm
              defaultModelUri={modelSchemaToUrl(model)}
              managementAddr={managementAddr}
            />
          </InferenceParametersContextProvider>
        </ChatTemplateContextProvider>
      );
    },
  });
}
