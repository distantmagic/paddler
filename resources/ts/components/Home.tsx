import React from "react";
import { Route, Router, Switch } from "wouter";

import { ChangeModelPage } from "./ChangeModelPage";
import { DashboardPage } from "./DashboardPage";
import { PromptContextProvider } from "./PromptContextProvider";
import { PromptPage } from "./PromptPage";
import { WorkbenchLayout } from "./WorkbenchLayout";

export function Home() {
  return (
    <Router>
      <WorkbenchLayout>
        <Switch>
          <Route path="/">
            <DashboardPage />
          </Route>
          <Route path="/model">
            <ChangeModelPage />
          </Route>
          <Route path="/prompt">
            <PromptContextProvider>
              <PromptPage />
            </PromptContextProvider>
          </Route>
          <Route>404 :(</Route>
        </Switch>
      </WorkbenchLayout>
    </Router>
  );
}
