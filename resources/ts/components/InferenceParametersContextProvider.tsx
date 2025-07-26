import React, { useMemo, useState, type ReactNode } from "react";

import { type InferenceParameters } from "../InferenceParameters.type";
import {
  InferenceParametersContext,
  type InferenceParametersContextValue,
} from "../contexts/InferenceParametersContext";

const defaultInferenceParameters: InferenceParameters = Object.freeze({
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

export function InferenceParametersContextProvider({
  children,
}: {
  children: ReactNode;
}) {
  const [parameters, setParameters] = useState<InferenceParameters>(
    defaultInferenceParameters,
  );

  const value = useMemo<InferenceParametersContextValue>(
    function () {
      function setPartialParameters(
        partialParameters: Partial<InferenceParameters>,
      ) {
        setParameters({
          ...parameters,
          ...partialParameters,
        });
      }

      function setParameter(name: keyof InferenceParameters, value: number) {
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
    <InferenceParametersContext.Provider value={value}>
      {children}
    </InferenceParametersContext.Provider>
  );
}
