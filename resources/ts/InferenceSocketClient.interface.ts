export interface InferenceSocketClient {
  generateTokens(params: {
    abortSignal: AbortSignal;
    onToken(this: void, token: string): void;
    prompt: string;
  }): void;
}
