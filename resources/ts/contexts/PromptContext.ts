import { createContext } from "react";

export type PromptContextValue = {
  currentPrompt: string;
  isCurrentPromptEmpty: boolean;
  setCurrentPrompt(this: void, prompt: string): void;
  setSubmittedPrompt(this: void, prompt: null | string): void;
  submittedPrompt: null | string;
  version: number;
};

export const PromptContext = createContext<PromptContextValue>({
  get currentPrompt(): never {
    throw new Error("PromptContext not provided");
  },
  get isCurrentPromptEmpty(): never {
    throw new Error("PromptContext not provided");
  },
  get submittedPrompt(): never {
    throw new Error("PromptContext not provided");
  },
  setCurrentPrompt(): never {
    throw new Error("PromptContext not provided");
  },
  setSubmittedPrompt(): never {
    throw new Error("PromptContext not provided");
  },
  get version(): never {
    throw new Error("PromptContext not provided");
  },
});
