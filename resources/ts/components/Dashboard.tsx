import clsx from "clsx";
import React, { useEffect, useState, CSSProperties } from "react";
import { z } from "zod";

import { DashboardLayout } from "./DashboardLayout";

const agentSchema = z.object({
  agent_id: z.string(),
  agent_name: z.string().nullable(),
  error: z.string().nullable(),
  external_llamacpp_addr: z.string(),
  is_authorized: z.boolean(),
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
            const hasIssues =
              agent.error || !agent.is_authorized || agent.quarantined_until;

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
                      <p>Error</p>
                      <p>{agent.error}</p>
                    </>
                  )}
                  {!agent.is_authorized && (
                    <>
                      <p>Unauthorized</p>
                      <p>
                        Probably llama.cpp API key is either invalid or not
                        present. Pass it to the agent with
                        `--llamacpp-api-key=YOURKEY` flag.
                      </p>
                    </>
                  )}
                  {agent.quarantined_until && (
                    <>
                      <p>
                        Quarantined until{" "}
                        {new Date(
                          agent.quarantined_until.secs_since_epoch * 1000,
                        ).toLocaleString()}
                      </p>
                    </>
                  )}
                  {!hasIssues && <p>None</p>}
                </td>
                <td>{agent.external_llamacpp_addr}</td>
                <td>
                  {new Date(
                    agent.last_update.secs_since_epoch * 1000,
                  ).toLocaleString()}
                </td>
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
