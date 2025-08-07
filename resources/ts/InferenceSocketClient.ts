import { nanoid } from "nanoid";
import { filter, fromEvent, map, takeWhile, type Observable } from "rxjs";

import { type ConversationMessage } from "./ConversationMessage.type";
import { type InferenceSocketClient } from "./InferenceSocketClient.interface";
import {
  InferenceServiceGenerateTokensResponseSchema,
  type InferenceServiceGenerateTokensResponse,
} from "./schemas/InferenceServiceGenerateTokensResponse";

export function InferenceSocketClient({
  webSocket,
}: {
  webSocket: WebSocket;
}): InferenceSocketClient {
  function continueConversation({
    conversation_history,
  }: {
    conversation_history: ConversationMessage[];
  }): Observable<InferenceServiceGenerateTokensResponse> {
    const requestId = nanoid();
    const messages = fromEvent<MessageEvent>(webSocket, "message").pipe(
      map(function (event): unknown {
        return event.data;
      }),
      filter(function (eventData) {
        return "string" === typeof eventData;
      }),
      map(function (serializedToken: string): unknown {
        return JSON.parse(serializedToken);
      }),
      map(function (parsedToken: unknown) {
        try {
          return InferenceServiceGenerateTokensResponseSchema.parse(
            parsedToken,
          );
        } catch (error: unknown) {
          console.error(
            "Failed to parse token response token:",
            parsedToken,
            error,
          );

          throw error;
        }
      }),
      filter(function ({ request_id }) {
        return request_id === requestId;
      }),
      takeWhile(function ({ done }) {
        return !done;
      }, true),
    );

    webSocket.send(
      JSON.stringify({
        Request: {
          id: requestId,
          request: {
            ContinueFromConversationHistory: {
              add_generation_prompt: true,
              enable_thinking: true,
              max_tokens: 1000,
              conversation_history,
            },
          },
        },
      }),
    );

    return messages;
  }

  return Object.freeze({
    continueConversation,
  });
}
