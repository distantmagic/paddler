import {
  type GenerateTokensResult,
  type InferenceSocketClient,
} from "./InferenceSocketClient.interface";

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
  }): GenerateTokensResult {
    const requestId = crypto.randomUUID();
    console.log(abortSignal);

    function onMessage(event: MessageEvent) {
      console.log("Message received:", event.data);
    }

    webSocket.addEventListener("message", onMessage);
    webSocket.send(
      JSON.stringify({
        id: requestId,
        request: {
          GenerateTokens: {
            prompt,
          },
        },
      }),
    );

    return Object.freeze({
      async *tokensStream() {
        await new Promise((resolve) => {
          resolve("xddd");
        });

        yield "xd yield";
      },
      [Symbol.dispose]() {
        console.log("Disposing generateTokens");
        webSocket.removeEventListener("message", onMessage);
      },
    });
  }

  return Object.freeze({
    generateTokens,
  });
}
