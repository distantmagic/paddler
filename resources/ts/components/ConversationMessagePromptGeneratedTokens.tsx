import React, { memo, useContext, useEffect, useMemo, useState } from "react";
import { scan } from "rxjs";

import { PromptContext } from "../contexts/PromptContext";
import { InferenceSocketClient } from "../InferenceSocketClient";
import { ConversationMessage } from "./ConversationMessage";

export const ConversationMessagePromptGeneratedTokens = memo(
  function ConversationMessagePromptGeneratedTokens({
    webSocket,
  }: {
    webSocket: WebSocket;
  }) {
    const { submittedPrompt } = useContext(PromptContext);
    const [message, setMessage] = useState<string>("");

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
            scan(function (message: string, token: string) {
              return `${message}${token}`;
            }),
          )
          .subscribe(setMessage);

        return function () {
          subscription.unsubscribe();
        };
      },
      [inferenceSocketClient, setMessage, submittedPrompt],
    );

    if (!message) {
      return;
    }

    return (
      <ConversationMessage>
        <strong>AI</strong>: {message}
      </ConversationMessage>
    );
  },
);
