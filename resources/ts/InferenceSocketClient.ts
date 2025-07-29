import { filter, fromEvent, map, takeWhile, type Observable } from "rxjs";

import { type ConversationMessage } from "./ConversationMessage.type";
import { type InferenceSocketClient } from "./InferenceSocketClient.interface";
import { InferenceServiceGenerateTokensResponseSchema } from "./schemas/InferenceServiceGenerateTokensResponse";

export function InferenceSocketClient({
  webSocket,
}: {
  webSocket: WebSocket;
}): InferenceSocketClient {
  function continueConversation({
    conversation_history,
  }: {
    conversation_history: ConversationMessage[];
  }): Observable<string> {
    const requestId = crypto.randomUUID();
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
        return InferenceServiceGenerateTokensResponseSchema.parse(parsedToken);
      }),
      filter(function ({ request_id }) {
        return request_id === requestId;
      }),
      takeWhile(function ({ done }) {
        return !done;
      }),
      map(function ({ token }): string {
        return String(token);
      }),
    );

    webSocket.send(
      JSON.stringify({
        Request: {
          id: requestId,
          request: {
            ContinueConversation: {
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
