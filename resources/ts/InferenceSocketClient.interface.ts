export interface InferenceSocketClient {
  generateTokens(params: {
    abortSignal: AbortSignal;
    onChunk(chunk: string): void;
    prompt: string;
  }): void;
}
