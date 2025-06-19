import clsx from "clsx";
import React, { useEffect, useState, CSSProperties } from "react";
import { z } from "zod";

import { DashboardLayout } from "./DashboardLayout";

const agentSchema = z.object({
  agent_id: z.string(),
  agent_name: z.string().nullable(),
  error: z.string().nullable(),
  is_llamacpp_reachable: z.boolean().nullable(),
  is_llamacpp_response_decodeable: z.boolean().nullable(),
  is_llamacpp_request_error: z.boolean().nullable(),
  external_llamacpp_addr: z.string(),
  is_authorized: z.boolean().nullable(),
  is_slots_endpoint_enabled: z.boolean().nullable(),
  last_update: z.object({
    nanos_since_epoch: z.number(),
    secs_since_epoch: z.number(),
  }),
  quarantined_until: z
    .object({
      nanos_since_epoch: z.number(),
      secs_since_epoch: z.number(),
    })
    .nullable(),
  slots_idle: z.number(),
  slots_processing: z.number(),
});

const agentsResponseSchema = z.object({
  agents: z.array(agentSchema),
});

// use zod just for the sake of integrity
type Agent = z.infer<typeof agentSchema>;
type AgentsResponse = z.infer<typeof agentsResponseSchema>;

const TICK_MS = 500;

function formatTimestamp(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleString();
}

export function Dashboard() {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [isError, setIsError] = useState(false);
  const [isFirstLoad, setIsFirstLoad] = useState(true);
  const [currentTick, setCurrentTick] = useState(0);

  useEffect(
    function () {
      const intervalId = setInterval(function () {
        setCurrentTick(function (previousTick) {
          return previousTick + 1;
        });
      }, TICK_MS);

      return function () {
        clearInterval(intervalId);
      };
    },
    [setCurrentTick],
  );

  useEffect(
    function () {
      const abortController = new AbortController();

      fetch("/api/v1/agents", {
        signal: abortController.signal,
      })
        .then((response) => response.json())
        .then((agents) => agentsResponseSchema.parse(agents))
        .then(function (agentsResponse: AgentsResponse) {
          setIsError(false);
          setAgents(agentsResponse.agents);
        })
        .catch(function (error) {
          setIsError(true);
          console.error(error);
        })
        .finally(function () {
          setIsFirstLoad(false);
        });

      return function () {
        // abort controller prevents overlapping requests
        abortController.abort();
      };
    },
    [
      // fetch new data every tick
      currentTick,
      setAgents,
      setIsError,
      setIsFirstLoad,
    ],
  );

  if (isError) {
    return (
      <DashboardLayout currentTick={currentTick}>
        <p>
          Error while fetching current agents from the management server. Is it
          running?
        </p>
        <p>Will automatically retry in a sec...</p>
      </DashboardLayout>
    );
  }

  if (isFirstLoad) {
    return (
      <DashboardLayout currentTick={currentTick}>
        <p>Loading agents...</p>
      </DashboardLayout>
    );
  }

  if (agents.length < 1) {
    return (
      <DashboardLayout currentTick={currentTick}>
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
    <DashboardLayout currentTick={currentTick}>
      <h1>Paddler 🏓</h1>
      <h2>Registered Agents</h2>
      <table>
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
            console.log(agent)
            const hasIssues =
              agent.error != null ||
              agent.is_authorized === false ||
              agent.is_slots_endpoint_enabled === false ||
              agent.is_llamacpp_reachable === false ||
              agent.is_llamacpp_response_decodeable === false ||
              agent.is_llamacpp_request_error === true ||
              agent.quarantined_until != null;
          
            console.log(hasIssues)
            return (
              <tr
                className={clsx("agent-row", {
                  "agent-row--error": hasIssues,
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
                  {false == agent.is_llamacpp_reachable && (
                      <p>Llama.cpp server is unreachable. It is likely down.</p>
                  )}
                  {false == agent.is_llamacpp_response_decodeable && (
                      <p>Llama.cpp server returned an unexpected response. Are your sure that agent observers llama.cpp, and does that at a correct port?</p>
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
                    <>
                      <p>
                        Quarantined until{" "}
                        {formatTimestamp(
                          agent.quarantined_until.secs_since_epoch,
                        )}
                      </p>
                    </>
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
                  className="agent-usage"
                  style={
                    {
                      "--slots-usage": `${(agent.slots_processing / (agent.slots_idle + agent.slots_processing)) * 100}%`,
                    } as CSSProperties
                  }
                >
                  <div className="agent-usage__progress"></div>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </DashboardLayout>
  );
}
