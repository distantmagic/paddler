import clsx from "clsx";
import React, { CSSProperties, useEffect, useState } from "react";
import { z } from "zod";

import { AgentSchema, type Agent } from "../schemas/Agent";
import { DashboardLayout } from "./DashboardLayout";

import {
  agentRow,
  agentRowError,
  agentUsage,
  agentUsage__progress,
  agentsTable,
} from "./Dashboard.module.css";

const AgentsResponseSchema = z.object({
  agents: z.array(AgentSchema),
});

function formatTimestamp(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleString();
}

export function Dashboard() {
  const [agents, setAgents] = useState<undefined | Agent[]>(undefined);
  const [isConnectionError, setIsConnectionError] = useState(false);
  const [isDeserializationError, setIsDeserializationError] = useState(false);

  useEffect(
    function () {
      const agentsSource = new EventSource("/api/v1/agents/stream");

      agentsSource.addEventListener("error", function () {
        setIsConnectionError(true);
      });

      agentsSource.addEventListener("message", function (event) {
        if ("string" !== typeof event.data) {
          console.error("Received non-string data from SSE:", event.data);
          setIsDeserializationError(true);

          return;
        }

        const parsed = JSON.parse(event.data);
        const result = AgentsResponseSchema.safeParse(parsed);

        if (!result.success) {
          setIsDeserializationError(true);
        } else {
          setAgents(result.data.agents);
        }
      });

      agentsSource.addEventListener("open", function () {
        setIsConnectionError(false);
        setIsDeserializationError(false);
      });

      return function () {
        agentsSource.close();
      };
    },
    [setAgents, setIsConnectionError, setIsDeserializationError],
  );

  if (isConnectionError) {
    return (
      <DashboardLayout>
        <p>
          Error while fetching current agents from the management server. Is it
          running?
        </p>
        <p>Will automatically retry in a sec...</p>
      </DashboardLayout>
    );
  }

  if (isDeserializationError) {
    return (
      <DashboardLayout>
        <p>Error while parsing agents data from the management server.</p>
        <p>Will automatically retry in a sec...</p>
      </DashboardLayout>
    );
  }

  if ("undefined" === typeof agents) {
    return (
      <DashboardLayout>
        <p>Loading agents...</p>
      </DashboardLayout>
    );
  }

  if (agents.length < 1) {
    return (
      <DashboardLayout>
        <h1>Paddler 🏓</h1>
        <h2>Registered Agents</h2>
        <p>No agents registered yet.</p>
        <p>
          If you have an agent running, please wait a few seconds for it to
          register itself.
        </p>
      </DashboardLayout>
    );
  }

  return (
    <DashboardLayout>
      <h1>Paddler 🏓</h1>
      <h2>Registered Agents</h2>
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
                  {false === agent.is_slots_endpoint_enabled && (
                    <>
                      <p>Slots endpoint is not enabled</p>
                      <p>
                        Probably llama.cpp server is running without the
                        `--slots` flag.
                      </p>
                    </>
                  )}
                  {agent.quarantined_until && (
                    <p>
                      Quarantined until{" "}
                      {formatTimestamp(
                        agent.quarantined_until.secs_since_epoch,
                      )}
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
    </DashboardLayout>
  );
}
