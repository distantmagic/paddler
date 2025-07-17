import React from "react";

import { useWebSocket } from "../hooks/useWebSocket";
import { matchWebSocketState } from "../matchWebSocketState";
import { webSocketProtocol } from "../webSocketProtocol";
import { ConversationMessage } from "./ConversationMessage";
import { FloatingStatus } from "./FloatingStatus";

export function ConversationMessagePromptGeneratedTokens({
  inferenceAddr,
  prompt,
}: {
  inferenceAddr: string;
  prompt: string;
}) {
  console.log(inferenceAddr, prompt);
  const webSocketState = useWebSocket({
    endpoint: `${webSocketProtocol(window.location.protocol)}//${inferenceAddr}/api/v1/inference_socket`,
  });

  return matchWebSocketState(webSocketState, {
    connected() {
      return (
        <ConversationMessage>
          <strong>Prompt Generated Tokens</strong>: This is a message generated
          by the prompt system.
        </ConversationMessage>
      );
    },
    connecting() {
      return (
        <FloatingStatus>Connecting to the inference server...</FloatingStatus>
      );
    },
    connectionClosed() {
      return (
        <FloatingStatus>
          Connection to the inference server closed. Will try to reconnect...
        </FloatingStatus>
      );
    },
    connectionError() {
      return (
        <FloatingStatus>
          Cannot connect to the inference server. Will try again in a moment...
        </FloatingStatus>
      );
    },
  });
}
