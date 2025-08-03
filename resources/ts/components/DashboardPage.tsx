import React from "react";

import { AgentListStream } from "./AgentListStream";
import { BufferedRequestsStream } from "./BufferedRequestsStream";
import { DashboardPageAddAgent } from "./DashboardPageAddAgent";

import {
  dashboardPage,
  dashboardPage__addrOverview,
  dashboardPage__blocks,
  dashboardPage__genericAddr,
  dashboardPage__inferenceAddr,
  dashboardPage__serviceBlock,
} from "./DashboardPage.module.css";

export function DashboardPage({
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
    <div className={dashboardPage}>
      <div className={dashboardPage__blocks}>
        <div className={dashboardPage__serviceBlock}></div>
        <div className={dashboardPage__serviceBlock}></div>
        <div className={dashboardPage__serviceBlock}>
          <div className={dashboardPage__genericAddr}>
            <p>Management addr: {managementAddr}</p>
          </div>
        </div>
      </div>
      <div className={dashboardPage__blocks}>
        <div className={dashboardPage__serviceBlock}>
          <div className={dashboardPage__addrOverview}>
            <div
              className={`${dashboardPage__genericAddr} ${dashboardPage__inferenceAddr}`}
            >
              <p>Inference addr: {inferenceAddr}</p>
            </div>
            {statsdAddr && (
              <div className={dashboardPage__genericAddr}>
                <p>StatsD addr: {statsdAddr}</p>
                <p>StatsD prefix: {statsdPrefix}</p>
                <p>
                  StatsD reporting interval:{" "}
                  {statsdReportingIntervalMillis / 1000}s
                </p>
              </div>
            )}
          </div>
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
