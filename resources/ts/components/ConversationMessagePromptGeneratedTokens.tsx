import React, { memo, useContext, useEffect, useMemo, useState } from "react";
import { scan } from "rxjs";

import { PromptContext } from "../contexts/PromptContext";
import { InferenceSocketClient } from "../InferenceSocketClient";
import { ConversationMessage } from "./ConversationMessage";

interface Message {
  isEmpty: boolean;
  isThinking: boolean;
  response: string;
  thoughts: string;
}

const defaultMessage: Message = Object.freeze({
  isEmpty: true,
  isThinking: false,
  response: "",
  thoughts: "",
});

export const ConversationMessagePromptGeneratedTokens = memo(
  function ConversationMessagePromptGeneratedTokens({
    webSocket,
  }: {
    webSocket: WebSocket;
  }) {
    const { submittedPrompt } = useContext(PromptContext);
    const [message, setMessage] = useState<Message>(defaultMessage);

    const inferenceSocketClient = useMemo(
      function () {
        return InferenceSocketClient({ webSocket });
      },
      [webSocket],
    );

    useEffect(
      function () {
        if (!submittedPrompt || !submittedPrompt.trim()) {
          return;
        }

        const subscription = inferenceSocketClient
          .continueConversation({
            conversation_history: [
              {
                role: "system",
                content:
                  "You are a helpful assistant. Give short, precise answers.",
              },
              { role: "user", content: submittedPrompt },
            ],
          })
          .pipe(
            scan(function (message: Message, token: string) {
              if ("<think>" === token) {
                return Object.freeze({
                  isEmpty: false,
                  isThinking: true,
                  response: message.response,
                  thoughts: message.thoughts,
                });
              }

              if ("</think>" === token) {
                return Object.freeze({
                  isEmpty: false,
                  isThinking: false,
                  response: message.response,
                  thoughts: message.thoughts,
                });
              }

              if (message.isThinking) {
                return Object.freeze({
                  isEmpty: false,
                  isThinking: true,
                  response: message.response,
                  thoughts: `${message.thoughts}${token}`,
                });
              }

              return Object.freeze({
                isEmpty: false,
                isThinking: false,
                response: `${message.response}${token}`,
                thoughts: message.thoughts,
              });
            }, defaultMessage),
          )
          .subscribe(setMessage);

        return function () {
          subscription.unsubscribe();
        };
      },
      [inferenceSocketClient, setMessage, submittedPrompt],
    );

    if (message.isEmpty) {
      return;
    }

    return (
      <ConversationMessage
        author="AI"
        isThinking={message.isThinking}
        response={message.response}
        thoughts={message.thoughts}
      />
    );
  },
);
