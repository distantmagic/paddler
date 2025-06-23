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
  console.log("agents", agents);
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
        {agents.map(function (agent: Agent) {
          const hasIssues =
            agent.error ||
            true !== agent.is_authorized ||
            true === agent.is_connect_error ||
            true === agent.is_request_error ||
            true === agent.is_decode_error ||
            true === agent.is_deserialize_error ||
            true === agent.is_unexpected_response_status ||
            true !== agent.is_slots_endpoint_enabled ||
            agent.quarantined_until;

          return (
            <tr
              className={clsx(agentRow, {
                [agentRowError]: hasIssues,
              })}
              key={agent.agent_id}
            >
              <td>{agent.agent_name}</td>
              <td>
                {agent.error && (
                  <>
                    <p>Agent reported an Error</p>
                    <p>{agent.error}</p>
                  </>
                )}
                {false === agent.is_authorized && (
                  <>
                    <p>Unauthorized</p>
                    <p>
                      Probably llama.cpp API key is either invalid or not
                      present. Pass it to the agent with
                      `--llamacpp-api-key=YOURKEY` flag.
                    </p>
                  </>
                )}
                {true == agent.is_connect_error && (
                  <p>Llama.cpp server is unreachable. It is likely down.</p>
                )}
                {true == agent.is_decode_error && (
                  <p>
                    Llama.cpp server returned an unexpected response. Are you
                    sure that the agent is configured to monitor llama.cpp and
                    is using the correct port?
                  </p>
                )}
                {true == agent.is_deserialize_error && (
                  <p>Llama.cpp server response could not be deserialized.</p>
                )}
                {true == agent.is_unexpected_response_status && (
                  <p>Llama.cpp server response status is unexpected.</p>
                )}
                {false === agent.is_slots_endpoint_enabled && (
                  <>
                    <p>Slots endpoint is not enabled</p>
                    <p>
                      Probably llama.cpp server is running without the `--slots`
                      flag.
                    </p>
                  </>
                )}
                {agent.quarantined_until && (
                  <p>
                    Quarantined until{" "}
                    {formatTimestamp(agent.quarantined_until.secs_since_epoch)}
                  </p>
                )}
                {!hasIssues && <p>None</p>}
              </td>
              <td>
                <a href={`http://${agent.external_llamacpp_addr}`}>
                  {agent.external_llamacpp_addr}
                </a>
              </td>
              <td>{formatTimestamp(agent.last_update.secs_since_epoch)}</td>
              <td>{agent.slots_idle}</td>
              <td>{agent.slots_processing}</td>
              <td
                className={agentUsage}
                style={
                  {
                    "--slots-usage": `${(agent.slots_processing / (agent.slots_idle + agent.slots_processing)) * 100}%`,
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
