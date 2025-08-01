import React from "react";
import { useParams } from "wouter";

import { useChatTemplate } from "../hooks/useChatTemplate";
import { matchFetchJsonState } from "../matchFetchJsonState";
import { ChatTemplateContextProvider } from "./ChatTemplateContextProvider";
import { ChatTemplatePageEditor } from "./ChatTemplatePageEditor";
import { ChatTemplatesPageToolbar } from "./ChatTemplatesPageToolbar";
import { FloatingStatus } from "./FloatingStatus";

import {
  chatTemplatesPage__editor,
  chatTemplatesPage__toolbar,
} from "./ChatTemplatesPage.module.css";

export function ChatTemplatePageTemplateLoader({
  managementAddr,
}: {
  managementAddr: string;
}) {
  const { id } = useParams<{ id: string }>();
  const loadingState = useChatTemplate({
    chatTemplateId: id,
    managementAddr,
  });

  return matchFetchJsonState(loadingState, {
    empty() {
      return <FloatingStatus>No chat template selected</FloatingStatus>;
    },
    error({ error }) {
      return (
        <FloatingStatus>Error loading chat template: {error}</FloatingStatus>
      );
    },
    loading() {
      return <FloatingStatus>Loading chat template...</FloatingStatus>;
    },
    ok({
      response: {
        chat_template: { content, id, name },
      },
    }) {
      return (
        <ChatTemplateContextProvider
          defaultContent={content}
          defaultName={name}
          exists={true}
          id={id}
        >
          <div className={chatTemplatesPage__toolbar}>
            <ChatTemplatesPageToolbar managementAddr={managementAddr} />
          </div>
          <div className={chatTemplatesPage__editor}>
            <ChatTemplatePageEditor />
          </div>
        </ChatTemplateContextProvider>
      );
    },
  });
}
