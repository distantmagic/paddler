import React from "react";

import { useEventSourceUpdates } from "../hooks/useEventSourceUpdates";
import { matchEventSourceUpdateState } from "../matchEventSourceUpdateState";
import { AgentsResponseSchema } from "../schemas/AgentsResponse";
import { AgentsList } from "./AgentsList";
import { DashboardLayout } from "./DashboardLayout";

export function Dashboard({ managementAddr }: { managementAddr: string }) {
  const eventSourceUpdateState = useEventSourceUpdates({
    schema: AgentsResponseSchema,
    endpoint: `${managementAddr}/api/v1/agents/stream`,
  });

  return matchEventSourceUpdateState(eventSourceUpdateState, {
    connected() {
      return (
        <DashboardLayout>
          <h1>Paddler 🏓</h1>
          <h2>Connected to the server, waiting for agents...</h2>
        </DashboardLayout>
      );
    },
    connectionError() {
      return (
        <DashboardLayout>
          <h1>Paddler 🏓</h1>
          <h2>Cannot connect to the server. Will try again in a moment...</h2>
        </DashboardLayout>
      );
    },
    dataSnapshot({ data }) {
      return (
        <DashboardLayout>
          <h1>Paddler 🏓</h1>
          <h2>Registered Agents</h2>
          <AgentsList agents={data.agents} />
        </DashboardLayout>
      );
    },
    deserializationError() {
      return (
        <DashboardLayout>
          <h1>Paddler 🏓</h1>
          <h2>Error deserializing data from the server</h2>
        </DashboardLayout>
      );
    },
    initial() {
      return (
        <DashboardLayout>
          <h1>Paddler 🏓</h1>
          <h2>Connecting to the server...</h2>
        </DashboardLayout>
      );
    },
  });
}
