import React from "react";
import { Route, Router, Switch } from "wouter";

import { AgentListPage } from "./AgentListPage";
import { ChangeModelPage } from "./ChangeModelPage";
import { PromptContextProvider } from "./PromptContextProvider";
import { PromptPage } from "./PromptPage";
import { WorkbenchLayout } from "./WorkbenchLayout";

export function Home({
  inferenceAddr,
  managementAddr,
}: {
  inferenceAddr: string;
  managementAddr: string;
}) {
  return (
    <Router>
      <WorkbenchLayout>
        <Switch>
          <Route path="/">
            <AgentListPage managementAddr={managementAddr} />
          </Route>
          <Route path="/model">
            <ChangeModelPage managementAddr={managementAddr} />
          </Route>
          <Route path="/prompt">
            <PromptContextProvider>
              <PromptPage inferenceAddr={inferenceAddr} />
            </PromptContextProvider>
          </Route>
          <Route>404 :(</Route>
        </Switch>
      </WorkbenchLayout>
    </Router>
  );
}
