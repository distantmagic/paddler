import { type Observable } from "rxjs";

export interface InferenceSocketClient {
  generateTokens(params: { prompt: string }): Observable<string>;
}
