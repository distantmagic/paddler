export interface InferenceSocketClient {
  generateTokens(params: {
    abortSignal: AbortSignal;
    prompt: string;
    onChunk(chunk: string): void;
  }): void;
}
