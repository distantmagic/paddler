import React from "react";
import { Route, Router, Switch } from "wouter";

import { AgentListPage } from "./AgentListPage";
import { ChatPage } from "./ChatPage";
import { DashboardLayout } from "./DashboardLayout";

export function Dashboard({ managementAddr }: { managementAddr: string }) {
  return (
    <Router>
      <DashboardLayout>
        <Switch>
          <Route path="/">
            <AgentListPage managementAddr={managementAddr} />
          </Route>
          <Route path="/chat" nest>
            <ChatPage />
          </Route>
          <Route>404 :(</Route>
        </Switch>
      </DashboardLayout>
    </Router>
  );
}
