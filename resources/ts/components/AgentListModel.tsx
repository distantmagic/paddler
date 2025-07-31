import React from "react";

import { ModelMetadataPreviewButton } from "./ModelMetadataPreviewButton";

import {
  agentListModel,
  agentListModel__name,
} from "./AgentListModel.module.css";

function displayLastPathPart(path: string | null): string {
  if (!path) {
    return "";
  }

  const parts = path.split("/");
  const last = parts.pop();

  if (!last) {
    return "";
  }

  return last;
}

export function AgentListModel({
  agentId,
  agentName,
  managementAddr,
  model_path,
}: {
  agentId: string;
  agentName: null | string;
  managementAddr: string;
  model_path: null | string;
}) {
  return (
    <div className={agentListModel}>
      {"string" === typeof model_path ? (
        <>
          <abbr className={agentListModel__name} title={model_path}>
            {displayLastPathPart(model_path)}
          </abbr>
          <ModelMetadataPreviewButton
            agentId={agentId}
            agentName={agentName}
            managementAddr={managementAddr}
          />
        </>
      ) : (
        <i>No model loaded</i>
      )}
    </div>
  );
}
