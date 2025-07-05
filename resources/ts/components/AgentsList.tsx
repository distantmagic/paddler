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

function formatTimestamp(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleString();
}

export function AgentsList({ agents }: { agents: Array<Agent> }) {
  return (
    <table className={agentsTable}>
      <thead>
        <tr>
          <th>Name</th>
          <th>Issues</th>
          <th>Llama.cpp address</th>
          <th>Last update</th>
          <th>Idle slots</th>
          <th>Processing slots</th>
        </tr>
      </thead>
      <tbody>
        {agents.map(function ({
          data: { agent_id, last_update, quarantined_until, status },
          meta: { has_issues },
        }: Agent) {
          return (
            <tr
              className={clsx(agentRow, {
                [agentRowError]: has_issues,
              })}
              key={agent_id}
            >
              <td>{status.agent_name}</td>
              <td>
                {status.error && (
                  <>
                    <p>Agent reported an Error</p>
                    <p>{status.error}</p>
                  </>
                )}
                {false === status.is_authorized && (
                  <>
                    <p>Unauthorized</p>
                    <p>
                      Probably llama.cpp API key is either invalid or not
                      present. Pass it to the agent with
                      `--llamacpp-api-key=YOURKEY` flag.
                    </p>
                  </>
                )}
                {true == status.is_connect_error && (
                  <p>Llama.cpp server is unreachable. It is likely down.</p>
                )}
                {true == status.is_decode_error && (
                  <p>
                    Llama.cpp server returned an unexpected response. Are you
                    sure that the agent is configured to monitor llama.cpp and
                    is using the correct port?
                  </p>
                )}
                {true == status.is_deserialize_error && (
                  <p>Llama.cpp server response could not be deserialized.</p>
                )}
                {true == status.is_unexpected_response_status && (
                  <p>Llama.cpp server response status is unexpected.</p>
                )}
                {false === status.is_slots_endpoint_enabled && (
                  <>
                    <p>Slots endpoint is not enabled</p>
                    <p>
                      Probably llama.cpp server is running without the `--slots`
                      flag.
                    </p>
                  </>
                )}
                {quarantined_until && (
                  <p>
                    Quarantined until{" "}
                    {formatTimestamp(quarantined_until.secs_since_epoch)}
                  </p>
                )}
                {!has_issues && <p>None</p>}
              </td>
              <td>
                <a href={`http://${status.external_llamacpp_addr}`}>
                  {status.external_llamacpp_addr}
                </a>
              </td>
              <td>{formatTimestamp(last_update.secs_since_epoch)}</td>
              <td>{status.slots_idle}</td>
              <td>{status.slots_processing}</td>
              <td
                className={agentUsage}
                style={
                  {
                    "--slots-usage": `${(status.slots_processing / (status.slots_idle + status.slots_processing)) * 100}%`,
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
