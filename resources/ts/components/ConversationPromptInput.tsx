import React from "react";

import iconArrowUpward from "../../icons/arrow_upward.svg";
import {
  conversationPromptInput,
  conversationPromptInput__button,
  conversationPromptInput__controls,
  conversationPromptInput__controls__track,
  conversationPromptInput__textarea,
} from "./ConversationPromptInput.module.css";

export function ConversationPromptInput() {
  return (
    <div className={conversationPromptInput}>
      <textarea
        autoFocus
        className={conversationPromptInput__textarea}
        placeholder="Type your prompt here..."
      />
      <div className={conversationPromptInput__controls}>
        <div className={conversationPromptInput__controls__track} />
        <div className={conversationPromptInput__controls__track}>
          <button className={conversationPromptInput__button}>
            <img src={iconArrowUpward} alt="Send" />
          </button>
        </div>
      </div>
    </div>
  );
}
