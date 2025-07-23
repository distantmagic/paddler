import { type InferenceSocketClient } from "./InferenceSocketClient.interface";
import { InferenceServiceGenerateTokensResponseSchema } from "./schemas/InferenceServiceGenerateTokensResponse";

export function InferenceSocketClient({
  webSocket,
}: {
  webSocket: WebSocket;
}): InferenceSocketClient {
  function generateTokens({
    abortSignal,
    onToken,
    prompt,
  }: {
    abortSignal: AbortSignal;
    onToken(this: void, token: string): void;
    prompt: string;
  }) {
    const requestId = crypto.randomUUID();

    console.log(abortSignal);

    function onMessage(event: MessageEvent) {
      if ("string" !== typeof event.data) {
        console.error("Received non-string data from WebSocket:", event.data);

        return;
      }

      const parsed = JSON.parse(event.data);
      const result =
        InferenceServiceGenerateTokensResponseSchema.safeParse(parsed);

      if (!result.success) {
        console.error(
          "Deserialization error:",
          event.data,
          result.error.issues,
        );
        return;
      } else {
        if (result.data.request_id !== requestId) {
          return;
        }

        if (result.data.done) {
          console.log("Done generating tokens for request:", requestId);
          webSocket.removeEventListener("message", onMessage);
        } else {
          onToken(result.data.token);
        }
      }
    }

    webSocket.addEventListener("message", onMessage);
    webSocket.send(
      JSON.stringify({
        Request: {
          id: requestId,
          request: {
            GenerateTokens: {
              max_tokens: 400,
              prompt,
            },
          },
        },
      }),
    );
  }

  return Object.freeze({
    generateTokens,
  });
}
