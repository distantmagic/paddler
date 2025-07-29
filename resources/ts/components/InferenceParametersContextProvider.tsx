import React, { useMemo, useState, type ReactNode } from "react";

import {
  InferenceParametersContext,
  type InferenceParametersContextValue,
} from "../contexts/InferenceParametersContext";
import { type InferenceParameters } from "../schemas/InferenceParameters";

export function InferenceParametersContextProvider({
  children,
  defaultInferenceParameters,
}: {
  children: ReactNode;
  defaultInferenceParameters: InferenceParameters;
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
