import React, { type ReactNode } from "react";

import { conversationMessage } from "./ConversationMessage.module.css";

export function ConversationMessage({ children }: { children: ReactNode }) {
  return <div className={conversationMessage}>{children}</div>;
}
