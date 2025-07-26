import React, { CSSProperties } from "react";

import { type Agent } from "../schemas/Agent";
import { AgentListModelDetailsButton } from "./AgentListModelDetailsButton";

import {
  agentList,
  agentList__model,
  agentList__progress,
  agentsTable,
} from "./AgentList.module.css";

export function AgentsList({
  agents,
  managementAddr,
}: {
  agents: Array<Agent>;
  managementAddr: string;
}) {
  return (
    <table className={agentsTable}>
      <thead>
        <tr>
          <th>Name</th>
          <th>Model</th>
          <th>Slots usage</th>
          <th>Used/Actual/Desired</th>
        </tr>
      </thead>
      <tbody>
        {agents.map(function ({
          id,
          desired_slots_total,
          model_path,
          name,
          slots_processing,
          slots_total,
        }: Agent) {
          return (
            <tr key={id}>
              <td>{name}</td>
              <td>
                {"string" === typeof model_path ? (
                  <div className={agentList__model}>
                    🪺
                    <AgentListModelDetailsButton
                      agentId={id}
                      managementAddr={managementAddr}
                      modelPath={model_path}
                    />
                  </div>
                ) : (
                  <div className={agentList__model}>
                    🪹 <i>No model loaded</i>
                  </div>
                )}
              </td>
              <td
                className={agentList}
                style={
                  {
                    "--slots-usage": `${((slots_total - slots_processing) / slots_total) * 100}%`,
                  } as CSSProperties
                }
              >
                {slots_total > 0 && <div className={agentList__progress}></div>}
              </td>
              <td>
                {slots_processing}/{slots_total}/{desired_slots_total}
              </td>
            </tr>
          );
        })}
      </tbody>
    </table>
  );
}
