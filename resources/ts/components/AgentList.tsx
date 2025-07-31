import React from "react";

import { type Agent } from "../schemas/Agent";
import { AgentListAgentStatus } from "./AgentListAgentStatus";
import { AgentListModel } from "./AgentListModel";

import {
  agentList,
  agentList__agent,
  agentList__agent__model,
  agentList__agent__name,
  agentList__agent__status,
} from "./AgentList.module.css";

export function AgentList({
  agents,
  managementAddr,
}: {
  agents: Array<Agent>;
  managementAddr: string;
}) {
  return (
    <div className={agentList}>
      {agents.map(function (agent: Agent) {
        const { id, name, model_path } = agent;

        return (
          <div className={agentList__agent} key={id}>
            <div className={agentList__agent__name}>{name}</div>
            <div className={agentList__agent__model}>
              <AgentListModel
                agentId={id}
                agentName={name}
                managementAddr={managementAddr}
                model_path={model_path}
              />
            </div>
            <div className={agentList__agent__status}>
              <AgentListAgentStatus agent={agent} />
            </div>
          </div>
        );
      })}
    </div>
  );
}
