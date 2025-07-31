import React from "react";

import { AgentListStream } from "./AgentListStream";
import { BufferedRequestsStream } from "./BufferedRequestsStream";
import { DashboardPageAddAgent } from "./DashboardPageAddAgent";
import { DashboardPageAddrOverview } from "./DashboardPageAddrOverview";

import {
  dashboardPage,
  dashboardPage__blocks,
  dashboardPage__serviceBlock,
} from "./DashboardPage.module.css";

export function DashboardPage({
  bufferedRequestTimeoutMillis,
  inferenceAddr,
  managementAddr,
  maxBufferedRequests,
}: {
  bufferedRequestTimeoutMillis: number;
  inferenceAddr: string;
  managementAddr: string;
  maxBufferedRequests: number;
}) {
  return (
    <div className={dashboardPage}>
      <div className={dashboardPage__blocks}>
        <div className={dashboardPage__serviceBlock}>
          <DashboardPageAddrOverview
            inferenceAddr={inferenceAddr}
            managementAddr={managementAddr}
          />
        </div>
        <div className={dashboardPage__serviceBlock}>
          <BufferedRequestsStream
            bufferedRequestTimeoutMillis={bufferedRequestTimeoutMillis}
            managementAddr={managementAddr}
            maxBufferedRequests={maxBufferedRequests}
          />
        </div>
        <div className={dashboardPage__serviceBlock}>
          <AgentListStream managementAddr={managementAddr} />
          <DashboardPageAddAgent managementAddr={managementAddr} />
        </div>
      </div>
    </div>
  );
}
