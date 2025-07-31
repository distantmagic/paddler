import React from "react";

import { AgentListStream } from "./AgentListStream";
import { DashboardBufferedRequests } from "./DashboardBufferedRequests";
import { DashboardPageAddAgent } from "./DashboardPageAddAgent";
import { DashboardPageAddrOverview } from "./DashboardPageAddrOverview";

import {
  dashboardPage,
  dashboardPage__blocks,
  dashboardPage__serviceBlock,
} from "./DashboardPage.module.css";

export function DashboardPage({
  bufferedRequestTimeoutMilis,
  inferenceAddr,
  managementAddr,
  maxBufferedRequests,
}: {
  bufferedRequestTimeoutMilis: number;
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
          <DashboardBufferedRequests
            bufferedRequestTimeoutMilis={bufferedRequestTimeoutMilis}
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
