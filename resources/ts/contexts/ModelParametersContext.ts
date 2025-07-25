import { createContext } from "react";

import { type ModelParameters } from "../ModelParameters.type";

export type ModelParametersContextValue = {
  parameters: ModelParameters;
  setParameter(this: void, name: keyof ModelParameters, value: number): void;
  setPartialParameters(this: void, parameters: Partial<ModelParameters>): void;
};

export const ModelParametersContext =
  createContext<ModelParametersContextValue>({
    get parameters(): never {
      throw new Error("ModelParametersContext not provided");
    },
    setParameter(): never {
      throw new Error("ModelParametersContext not provided");
    },
    setPartialParameters(): never {
      throw new Error("ModelParametersContext not provided");
    },
  });
