import { createContext } from "react";

export type ModelParametersContextValue = {
  isCurrentPromptEmpty: boolean;
  setCurrentPrompt(this: void, prompt: string): void;
};

export const ModelParametersContext =
  createContext<ModelParametersContextValue>({
    get isCurrentPromptEmpty(): never {
      throw new Error("ModelParametersContext not provided");
    },
    setCurrentPrompt(): never {
      throw new Error("ModelParametersContext not provided");
    },
  });
