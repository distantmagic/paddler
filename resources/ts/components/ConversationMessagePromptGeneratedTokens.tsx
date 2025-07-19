import React, { useEffect, useState } from "react";

import { type InferenceSocketClient } from "../InferenceSocketClient.interface";
import { ConversationMessage } from "./ConversationMessage";

export function ConversationMessagePromptGeneratedTokens({
  inferenceSocketClient,
  prompt,
}: {
  inferenceSocketClient: InferenceSocketClient;
  prompt: string;
}) {
  const [message, setMessage] = useState<string>("");

  useEffect(
    function () {
      const abortController = new AbortController();

      inferenceSocketClient.generateTokens({
        abortSignal: abortController.signal,
        prompt,
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
    [inferenceSocketClient, prompt, setMessage],
  );

  return (
    <ConversationMessage>
      <strong>Prompt Generated Tokens</strong>: {message}
    </ConversationMessage>
  );
}
