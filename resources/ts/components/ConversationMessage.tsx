import React from "react";
import Markdown from "react-markdown";

import {
  conversationMessage,
  conversationMessage__author,
  conversationMessage__response,
  conversationMessage__thoughts,
} from "./ConversationMessage.module.css";

export function ConversationMessage({
  author,
  isThinking,
  response,
  thoughts,
}: {
  author: string;
  isThinking: boolean;
  response: string;
  thoughts: string;
}) {
  return (
    <div className={conversationMessage}>
      <strong className={conversationMessage__author}>{author}:</strong>
      <div className={conversationMessage__response}>
        <div>{isThinking ? "🤔" : <Markdown>{response}</Markdown>}</div>
      </div>
      <div className={conversationMessage__thoughts}>{thoughts}</div>
    </div>
  );
}
