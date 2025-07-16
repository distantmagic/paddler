import React, { useContext } from "react";

import { PromptContext } from "../contexts/PromptContext";
import { ConversationMessagePromptGeneratedTokens } from "./ConversationMessagePromptGeneratedTokens";
import { ConversationPromptInput } from "./ConversationPromptInput";

import {
  promptPage,
  promptPage__messages,
  promptPage__promptForm,
} from "./PromptPage.module.css";

export function PromptPage({ inferenceAddr }: { inferenceAddr: string }) {
  const { submittedPrompt } = useContext(PromptContext);

  return (
    <div className={promptPage}>
      <div className={promptPage__messages}>
        {submittedPrompt && (
          <ConversationMessagePromptGeneratedTokens
            inferenceAddr={inferenceAddr}
            prompt={submittedPrompt}
          />
        )}
      </div>
      <div className={promptPage__promptForm}>
        <ConversationPromptInput />
      </div>
    </div>
  );
}
