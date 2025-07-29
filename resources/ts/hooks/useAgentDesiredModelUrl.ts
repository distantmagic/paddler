import { useMemo, useState } from "react";

import { type AgentDesiredModel } from "../schemas/AgentDesiredModel";
import { urlToAgentDesiredModel } from "../urlToAgentDesiredModel";

type EmptyState = {
  agentDesiredModel: null;
  empty: true;
  error: null;
  ok: false;
};

type ErrorState = {
  agentDesiredModel: null;
  empty: false;
  error: string;
  ok: false;
};

type SuccessState = {
  agentDesiredModel: AgentDesiredModel;
  empty: false;
  error: null;
  ok: true;
};

export type AgentDesiredModelState = EmptyState | ErrorState | SuccessState;

const emptyState: EmptyState = Object.freeze({
  agentDesiredModel: null,
  empty: true,
  error: null,
  ok: false,
});

export function useAgentDesiredModelUrl({
  defaultModelUri,
}: {
  defaultModelUri: null | string;
}): {
  agentDesiredModelState: AgentDesiredModelState;
  modelUri: null | string;
  setModelUri(this: void, modelUri: null | string): void;
} {
  const [modelUri, setModelUri] = useState<null | string>(defaultModelUri);

  const agentDesiredModelState: AgentDesiredModelState = useMemo(
    function () {
      if (!modelUri) {
        return emptyState;
      }

      try {
        return Object.freeze({
          agentDesiredModel: urlToAgentDesiredModel(new URL(modelUri)),
          empty: false,
          error: null,
          ok: true,
        });
      } catch (error: unknown) {
        return Object.freeze({
          agentDesiredModel: null,
          empty: false,
          error: String(error),
          ok: false,
        });
      }
    },
    [modelUri],
  );

  return Object.freeze({
    agentDesiredModelState,
    modelUri,
    setModelUri,
  });
}
