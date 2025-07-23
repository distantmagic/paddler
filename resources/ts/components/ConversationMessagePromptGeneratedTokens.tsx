import React, { memo, useContext, useEffect, useMemo, useState } from "react";

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

        const abortController = new AbortController();

        setMessage("");

        inferenceSocketClient.generateTokens({
          abortSignal: abortController.signal,
          prompt: `<|im_start|>system
          You are a helpful assistant. Give short, precise answers.<|im_end|>
          <|im_start|>user
          ${submittedPrompt}<|im_end|>
          <|im_start|>assistant`,
          onToken(token: string) {
            setMessage(function (prevMessage) {
              return `${prevMessage}${token}`;
            });
          },
        });

        return function () {
          abortController.abort();
        };
      },
      [inferenceSocketClient, setMessage, submittedPrompt],
    );

    return (
      <ConversationMessage>
        <strong>Prompt Generated Tokens</strong>: {message}
      </ConversationMessage>
    );
  },
);
