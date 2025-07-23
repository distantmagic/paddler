import React, { CSSProperties } from "react";

import { type Agent } from "../schemas/Agent";

import {
  agentUsage,
  agentUsage__progress,
  agentsTable,
} from "./AgentList.module.css";

function displayLastPathPart(path: string | null | undefined): string {
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

export function AgentsList({ agents }: { agents: Array<Agent> }) {
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
                <abbr title={model_path ?? undefined}>
                  {displayLastPathPart(model_path)}
                </abbr>
              </td>
              <td
                className={agentUsage}
                style={
                  {
                    "--slots-usage": `${((slots_total - slots_processing) / slots_total) * 100}%`,
                  } as CSSProperties
                }
              >
                {slots_total > 0 && (
                  <div className={agentUsage__progress}></div>
                )}
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
