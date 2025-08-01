import React from "react";

import { useEventSourceUpdates } from "../hooks/useEventSourceUpdates";
import { matchEventSourceUpdateState } from "../matchEventSourceUpdateState";
import { ChatTemplateHeadsResponseSchema } from "../schemas/ChatTemplateHeadsResponse";
import { ChatTemplatesPageTemplatesList } from "./ChatTemplatesPageTemplatesList";

import { chatTemplatesPageTemplatesStream__loader } from "./ChatTemplatesPageTemplatesStream.module.css";

export function ChatTemplatesPageTemplatesStream({
  managementAddr,
}: {
  managementAddr: string;
}) {
  const eventSourceUpdateState = useEventSourceUpdates({
    schema: ChatTemplateHeadsResponseSchema,
    endpoint: `//${managementAddr}/api/v1/chat_template_heads/stream`,
  });

  return matchEventSourceUpdateState(eventSourceUpdateState, {
    connected() {
      return (
        <div className={chatTemplatesPageTemplatesStream__loader}>
          Connected to the server, waiting for chat templates...
        </div>
      );
    },
    connectionError() {
      return (
        <div className={chatTemplatesPageTemplatesStream__loader}>
          Cannot connect to the server to get the chat templates. Will try to
          reconnect in a few seconds...
        </div>
      );
    },
    dataSnapshot({ data: { chat_template_heads } }) {
      return (
        <ChatTemplatesPageTemplatesList
          chat_template_heads={chat_template_heads}
        />
      );
    },
    deserializationError() {
      return (
        <div className={chatTemplatesPageTemplatesStream__loader}>
          Error deserializing buffered requests data from the server.
        </div>
      );
    },
    initial() {
      return (
        <div className={chatTemplatesPageTemplatesStream__loader}>
          Connecting to the server...
        </div>
      );
    },
  });
}
