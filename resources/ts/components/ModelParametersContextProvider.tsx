import React, { useMemo, useState, type ReactNode } from "react";

import { type ModelParameters } from "../ModelParameters.type";
import {
  ModelParametersContext,
  type ModelParametersContextValue,
} from "../contexts/ModelParametersContext";

const defaultModelParameters: ModelParameters = Object.freeze({
  batch_n_tokens: 512,
  context_size: 4096,
  min_p: 0.05,
  penalty_frequency: 0.0,
  penalty_last_n: -1,
  penalty_presence: 1.5,
  penalty_repeat: 1.0,
  temperature: 0.6,
  top_k: 40,
  top_p: 0.3,
});

export function ModelParametersContextProvider({
  children,
}: {
  children: ReactNode;
}) {
  const [parameters, setParameters] = useState<ModelParameters>(
    defaultModelParameters,
  );

  const value = useMemo<ModelParametersContextValue>(
    function () {
      function setPartialParameters(
        partialParameters: Partial<ModelParameters>,
      ) {
        setParameters({
          ...parameters,
          ...partialParameters,
        });
      }

      function setParameter(name: keyof ModelParameters, value: number) {
        setPartialParameters({
          ...parameters,
          ...{
            [name]: value,
          },
        });
      }

      return Object.freeze({
        parameters,
        setParameter,
        setPartialParameters,
      });
    },
    [parameters, setParameters],
  );

  return (
    <ModelParametersContext.Provider value={value}>
      {children}
    </ModelParametersContext.Provider>
  );
}
