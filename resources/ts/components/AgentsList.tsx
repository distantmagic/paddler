import clsx from "clsx";
import React, { CSSProperties } from "react";

import { type Agent } from "../schemas/Agent";

import {
  agentRow,
  agentRowError,
  agentUsage,
  agentUsage__progress,
  agentsTable,
} from "./Dashboard.module.css";

export function AgentsList({ agents }: { agents: Array<Agent> }) {
  return (
    <table className={agentsTable}>
      <thead>
        <tr>
          <th>Name</th>
          <th>Issues</th>
          <th>Slots usage</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        {agents.map(function ({
          data: { id, name, slots_processing, slots_total },
          meta: { has_issues },
        }: Agent) {
          return (
            <tr
              className={clsx(agentRow, {
                [agentRowError]: has_issues,
              })}
              key={id}
            >
              <td>{name}</td>
              <td>{!has_issues && <p>None</p>}</td>
              <td
                className={agentUsage}
                style={
                  {
                    "--slots-usage": `${((slots_total - slots_processing) / slots_total) * 100}%`,
                  } as CSSProperties
                }
              >
                <div className={agentUsage__progress}></div>
              </td>
              <td>
                {slots_processing}/{slots_total}
              </td>
            </tr>
          );
        })}
      </tbody>
    </table>
  );
}
