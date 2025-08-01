import React, { useMemo, useState, type ReactNode } from "react";

import {
  ChatTemplateContext,
  type ChatTemplateContextValue,
} from "../contexts/ChatTemplateContext";

export function ChatTemplateContextProvider({
  children,
  defaultContent,
  defaultName,
  exists,
  id: providedId,
}: {
  children: ReactNode;
  defaultContent: string;
  defaultName: string;
  exists: boolean;
  id?: string;
}) {
  const [content, setContent] = useState<string>(defaultContent);
  const [name, setName] = useState<string>(defaultName);

  const id = useMemo<string>(
    function () {
      if (providedId) {
        return providedId;
      }

      return crypto.randomUUID();
    },
    [providedId],
  );

  const value = useMemo<ChatTemplateContextValue>(
    function () {
      return Object.freeze({
        content,
        exists,
        id,
        name,
        setContent,
        setName,
      });
    },
    [content, exists, id, name, setContent, setName],
  );

  return (
    <ChatTemplateContext.Provider value={value}>
      {children}
    </ChatTemplateContext.Provider>
  );
}
