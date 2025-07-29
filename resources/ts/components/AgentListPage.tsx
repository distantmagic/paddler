import React from "react";

import { useEventSourceUpdates } from "../hooks/useEventSourceUpdates";
import { matchEventSourceUpdateState } from "../matchEventSourceUpdateState";
import { AgentsResponseSchema } from "../schemas/AgentsResponse";
import { AgentsList } from "./AgentsList";
import { FloatingStatus } from "./FloatingStatus";

import { agentListPage } from "./AgentListPage.module.css";

export function AgentListPage({ managementAddr }: { managementAddr: string }) {
  const eventSourceUpdateState = useEventSourceUpdates({
    schema: AgentsResponseSchema,
    endpoint: `//${managementAddr}/api/v1/agents/stream`,
  });

  return matchEventSourceUpdateState(eventSourceUpdateState, {
    connected() {
      return (
        <FloatingStatus>
          Connected to the server, waiting for agents...
        </FloatingStatus>
      );
    },
    connectionError() {
      return (
        <FloatingStatus>
          Cannot connect to the server. Will try again in a moment...
        </FloatingStatus>
      );
    },
    dataSnapshot({ data }) {
      if (data.agents.length < 1) {
        return <FloatingStatus>No agents registered yet.</FloatingStatus>;
      }

      return (
        <div className={agentListPage}>
          <AgentsList agents={data.agents} managementAddr={managementAddr} />
        </div>
      );
    },
    deserializationError() {
      return (
        <FloatingStatus>
          Error deserializing data from the server
        </FloatingStatus>
      );
    },
    initial() {
      return <FloatingStatus>Connecting to the server...</FloatingStatus>;
    },
  });
}
