import React from "react";
import { Route, Switch } from "wouter";

import { ChatTemplateContextProvider } from "./ChatTemplateContextProvider";
import { ChatTemplatePageEditor } from "./ChatTemplatePageEditor";
import { ChatTemplatePageTemplateLoader } from "./ChatTemplatePageTemplateLoader";
import { ChatTemplatesPageTemplatesStream } from "./ChatTemplatesPageTemplatesStream";
import { ChatTemplatesPageToolbar } from "./ChatTemplatesPageToolbar";

import {
  chatTemplatesPage,
  chatTemplatesPage__editor,
  chatTemplatesPage__templates,
  chatTemplatesPage__toolbar,
} from "./ChatTemplatesPage.module.css";

export function ChatTemplatesPage({
  managementAddr,
}: {
  managementAddr: string;
}) {
  return (
    <div className={chatTemplatesPage}>
      <div className={chatTemplatesPage__templates}>
        <ChatTemplatesPageTemplatesStream managementAddr={managementAddr} />
      </div>
      <Switch>
        <Route path="/:id">
          <ChatTemplatePageTemplateLoader managementAddr={managementAddr} />
        </Route>
        <Route>
          <ChatTemplateContextProvider
            defaultContent=""
            defaultName=""
            exists={false}
          >
            <div className={chatTemplatesPage__toolbar}>
              <ChatTemplatesPageToolbar managementAddr={managementAddr} />
            </div>
            <div className={chatTemplatesPage__editor}>
              <ChatTemplatePageEditor />
            </div>
          </ChatTemplateContextProvider>
        </Route>
      </Switch>
    </div>
  );
}
