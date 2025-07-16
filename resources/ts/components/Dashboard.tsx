import React from "react";
import { Route, Router, Switch } from "wouter";

import { AgentListPage } from "./AgentListPage";
import { DashboardLayout } from "./DashboardLayout";
import { PromptPage } from "./PromptPage";

export function Dashboard({ managementAddr }: { managementAddr: string }) {
  return (
    <Router>
      <DashboardLayout>
        <Switch>
          <Route path="/">
            <AgentListPage managementAddr={managementAddr} />
          </Route>
          <Route path="/prompt">
            <PromptPage />
          </Route>
          <Route>404 :(</Route>
        </Switch>
      </DashboardLayout>
    </Router>
  );
}
