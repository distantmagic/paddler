import { createContext } from "react";

export type ChatTemplateContextValue = {
  content: string;
  exists: boolean;
  id: string;
  name: string;
  setContent(this: void, content: string): void;
  setName(this: void, name: string): void;
};

export const ChatTemplateContext = createContext<ChatTemplateContextValue>({
  get content(): never {
    throw new Error("ChatTemplateContext not provided");
  },
  get exists(): never {
    throw new Error("ChatTemplateContext not provided");
  },
  get id(): never {
    throw new Error("ChatTemplateContext not provided");
  },
  get name(): never {
    throw new Error("ChatTemplateContext not provided");
  },
  setContent(): never {
    throw new Error("ChatTemplateContext not provided");
  },
  setName(): never {
    throw new Error("ChatTemplateContext not provided");
  },
});
