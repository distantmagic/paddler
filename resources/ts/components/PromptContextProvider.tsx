import React, { useEffect, useMemo, useState, type ReactNode } from "react";

import {
  PromptContext,
  type PromptContextValue,
} from "../contexts/PromptContext";

export function PromptContextProvider({ children }: { children: ReactNode }) {
  const [currentPrompt, setCurrentPrompt] = useState<string>("");
  const [submittedPrompt, setSubmittedPrompt] = useState<string | null>(null);

  useEffect(() => {
    setCurrentPrompt("");
  }, [submittedPrompt, setCurrentPrompt]);

  const value = useMemo<PromptContextValue>(
    function () {
      return Object.freeze({
        currentPrompt,
        isCurrentPromptEmpty: currentPrompt.trim() === "",
        setCurrentPrompt,
        setSubmittedPrompt,
        submittedPrompt,
      });
    },
    [currentPrompt, setCurrentPrompt, setSubmittedPrompt, submittedPrompt],
  );

  return (
    <PromptContext.Provider value={value}>{children}</PromptContext.Provider>
  );
}
