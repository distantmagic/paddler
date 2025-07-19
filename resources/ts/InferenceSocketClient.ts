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
        console.error("Deserialization error:", result.error.issues);
        return;
      } else {
        onToken(result.data.token);
      }
    }

    webSocket.addEventListener("message", onMessage);
    webSocket.addEventListener("message", function () {
      console.log("Received message from WebSocket");
    });
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
