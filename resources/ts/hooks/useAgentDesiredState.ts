import { useCallback } from "react";

import { AgentDesiredStateSchema } from "../schemas/AgentDesiredState";
import { useFetchJson } from "./useFetchJson";

export function useAgentDesiredState({
  managementAddr,
}: {
  managementAddr: string;
}) {
  const produceFetchPromise = useCallback(
    function (signal: AbortSignal) {
      return fetch(`//${managementAddr}/api/v1/agent_desired_state`, {
        signal,
      });
    },
    [managementAddr],
  );

  const result = useFetchJson({
    produceFetchPromise,
    responseSchema: AgentDesiredStateSchema,
  });

  return result;
}
