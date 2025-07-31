import React from "react";

import { AgentListBufferedRequests } from "./AgentListBufferedRequests";
import { AgentListStream } from "./AgentListStream";

import {
  agentListPage,
  agentListPage__serviceBlock,
} from "./AgentListPage.module.css";

export function AgentListPage({
  inferenceAddr,
  managementAddr,
}: {
  inferenceAddr: string;
  managementAddr: string;
}) {
  return (
    <div className={agentListPage}>
      <div className={agentListPage__serviceBlock}>
        <p>Inference addr: {inferenceAddr}</p>
        <p>Management addr: {managementAddr}</p>
      </div>
      <div className={agentListPage__serviceBlock}>
        <AgentListBufferedRequests />
      </div>
      <div className={agentListPage__serviceBlock}>
        <AgentListStream managementAddr={managementAddr} />
      </div>
    </div>
  );
}
