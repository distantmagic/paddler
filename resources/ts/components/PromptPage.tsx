import React from "react";

import { ConversationMessage } from "./ConversationMessage";
import { ConversationPromptInput } from "./ConversationPromptInput";

import {
  promptPage,
  promptPage__messages,
  promptPage__promptForm,
} from "./PromptPage.module.css";

export function PromptPage() {
  return (
    <div className={promptPage}>
      <div className={promptPage__messages}>
        {Array.from({ length: 5 }, function (_, i) {
          return (
            <ConversationMessage key={i}>
              <strong>User</strong>: This is a message number {i + 1}
            </ConversationMessage>
          );
        })}
      </div>
      <div className={promptPage__promptForm}>
        <ConversationPromptInput />
      </div>
    </div>
  );
}
