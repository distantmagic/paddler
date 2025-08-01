import React from "react";

import { useEventSourceUpdates } from "../hooks/useEventSourceUpdates";
import { matchEventSourceUpdateState } from "../matchEventSourceUpdateState";
import { AgentsResponseSchema } from "../schemas/AgentsResponse";
import { AgentList } from "./AgentList";

import { dashboardSectionStreamLoader } from "./dashboardSectionStreamLoader.module.css";

export function AgentListStream({
  managementAddr,
}: {
  managementAddr: string;
}) {
  const eventSourceUpdateState = useEventSourceUpdates({
    schema: AgentsResponseSchema,
    endpoint: `//${managementAddr}/api/v1/agents/stream`,
  });

  return matchEventSourceUpdateState(eventSourceUpdateState, {
    connected() {
      return (
        <div className={dashboardSectionStreamLoader}>
          Connected to the server, waiting for agents...
        </div>
      );
    },
    connectionError() {
      return (
        <div className={dashboardSectionStreamLoader}>
          Connecting to the server to get agents updates. Will try to reconnect
          in a few seconds...
        </div>
      );
    },
    dataSnapshot({ data: { agents } }) {
      if (agents.length < 1) {
        return <div>No agents registered yet.</div>;
      }

      return <AgentList agents={agents} managementAddr={managementAddr} />;
    },
    deserializationError() {
      return (
        <div className={dashboardSectionStreamLoader}>
          Error deserializing agents data from the server.
        </div>
      );
    },
    initial() {
      return (
        <div className={dashboardSectionStreamLoader}>
          Connecting to the server...
        </div>
      );
    },
  });
}
