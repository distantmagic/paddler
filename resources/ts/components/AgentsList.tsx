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

// function formatTimestamp(timestamp: number): string {
//   return new Date(timestamp * 1000).toLocaleString();
// }

export function AgentsList({ agents }: { agents: Array<Agent> }) {
  return (
    <table className={agentsTable}>
      <thead>
        <tr>
          <th>Name</th>
          <th>Issues</th>
          <th>Last update</th>
          <th>Slots</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        {agents.map(function ({
          data: { id, name },
          meta: { has_issues },
        }: Agent) {
          const slots_idle = 2;
          const slots_processing = 3;

          return (
            <tr
              className={clsx(agentRow, {
                [agentRowError]: has_issues,
              })}
              key={id}
            >
              <td>{name}</td>
              <td>{!has_issues && <p>None</p>}</td>
              {/* <td>{formatTimestamp(last_update.secs_since_epoch)}</td> */}
              <td>{slots_idle}</td>
              <td>{slots_processing}</td>
              <td
                className={agentUsage}
                style={
                  {
                    "--slots-usage": `${(slots_processing / (slots_idle + slots_processing)) * 100}%`,
                  } as CSSProperties
                }
              >
                <div className={agentUsage__progress}></div>
              </td>
            </tr>
          );
        })}
      </tbody>
    </table>
  );
}
