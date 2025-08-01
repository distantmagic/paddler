import React from "react";
import { Route, Router, Switch } from "wouter";

import { ChangeModelPage } from "./ChangeModelPage";
import { ChatTemplatesPage } from "./ChatTemplatesPage";
import { DashboardPage } from "./DashboardPage";
import { PromptContextProvider } from "./PromptContextProvider";
import { PromptPage } from "./PromptPage";
import { WorkbenchLayout } from "./WorkbenchLayout";

export function Home({
  bufferedRequestTimeoutMillis,
  inferenceAddr,
  managementAddr,
  maxBufferedRequests,
  statsdAddr,
  statsdPrefix,
  statsdReportingIntervalMillis,
}: {
  bufferedRequestTimeoutMillis: number;
  inferenceAddr: string;
  managementAddr: string;
  maxBufferedRequests: number;
  statsdAddr: string;
  statsdPrefix: string;
  statsdReportingIntervalMillis: number;
}) {
  return (
    <Router>
      <WorkbenchLayout>
        <Switch>
          <Route path="/">
            <DashboardPage
              bufferedRequestTimeoutMillis={bufferedRequestTimeoutMillis}
              inferenceAddr={inferenceAddr}
              managementAddr={managementAddr}
              maxBufferedRequests={maxBufferedRequests}
              statsdAddr={statsdAddr}
              statsdPrefix={statsdPrefix}
              statsdReportingIntervalMillis={statsdReportingIntervalMillis}
            />
          </Route>
          <Route path="/model">
            <ChangeModelPage managementAddr={managementAddr} />
          </Route>
          <Route path="/chat-templates">
            <ChatTemplatesPage managementAddr={managementAddr} />
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
