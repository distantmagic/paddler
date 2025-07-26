import { createContext } from "react";

import { type InferenceParameters } from "../InferenceParameters.type";

export type InferenceParametersContextValue = {
  parameters: InferenceParameters;
  setParameter(
    this: void,
    name: keyof InferenceParameters,
    value: number,
  ): void;
  setPartialParameters(
    this: void,
    parameters: Partial<InferenceParameters>,
  ): void;
};

export const InferenceParametersContext =
  createContext<InferenceParametersContextValue>({
    get parameters(): never {
      throw new Error("InferenceParametersContext not provided");
    },
    setParameter(): never {
      throw new Error("InferenceParametersContext not provided");
    },
    setPartialParameters(): never {
      throw new Error("InferenceParametersContext not provided");
    },
  });
