import React from "react";
import { Route, Router, Switch } from "wouter";

import { AgentListPage } from "./AgentListPage";
import { DashboardLayout } from "./DashboardLayout";
import { PromptContextProvider } from "./PromptContextProvider";
import { PromptPage } from "./PromptPage";

export function Dashboard({
  inferenceAddr,
  managementAddr,
}: {
  inferenceAddr: string;
  managementAddr: string;
}) {
  return (
    <Router>
      <DashboardLayout>
        <Switch>
          <Route path="/">
            <AgentListPage managementAddr={managementAddr} />
          </Route>
          <Route path="/prompt">
            <PromptContextProvider>
              <PromptPage inferenceAddr={inferenceAddr} />
            </PromptContextProvider>
          </Route>
          <Route>404 :(</Route>
        </Switch>
      </DashboardLayout>
    </Router>
  );
}
