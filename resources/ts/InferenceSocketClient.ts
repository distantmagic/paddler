import { type InferenceSocketClient } from "./InferenceSocketClient.interface";

export function InferenceSocketClient({
  webSocket,
}: {
  webSocket: WebSocket;
}): InferenceSocketClient {
  function generateTokens({
    abortSignal,
    prompt,
  }: {
    abortSignal: AbortSignal;
    prompt: string;
  }) {
    const requestId = crypto.randomUUID();
    console.log(abortSignal);

    function onMessage(event: MessageEvent) {
      console.log("Message received:", event.data);
    }

    webSocket.addEventListener("message", onMessage);
    webSocket.send(
      JSON.stringify({
        Request: {
          id: requestId,
          request: {
            GenerateTokens: {
              max_tokens: 1000,
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
