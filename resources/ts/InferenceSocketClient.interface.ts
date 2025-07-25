import { type Observable } from "rxjs";

import { type ConversationMessage } from "./ConversationMessage.type";

export interface InferenceSocketClient {
  continueConversation(params: {
    conversation_history: ConversationMessage[];
  }): Observable<string>;
}
