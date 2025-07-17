export interface GenerateTokensResult extends Disposable {
  tokensStream(): AsyncGenerator<string>;
}

export interface InferenceSocketClient {
  generateTokens(params: {
    abortSignal: AbortSignal;
    prompt: string;
  }): GenerateTokensResult;
}
