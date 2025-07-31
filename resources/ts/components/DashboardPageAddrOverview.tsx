import React from "react";

import {
  dashboardPageAddrOverview,
  dashboardPageAddrOverview__genericAddr,
  dashboardPageAddrOverview__inferenceAddr,
} from "./DashboardPageAddrOverview.module.css";

export function DashboardPageAddrOverview({
  inferenceAddr,
  managementAddr,
  statsdAddr,
  statsdPrefix,
  statsdReportingIntervalMillis,
}: {
  inferenceAddr: string;
  managementAddr: string;
  statsdAddr: string;
  statsdPrefix: string;
  statsdReportingIntervalMillis: number;
}) {
  return (
    <div className={dashboardPageAddrOverview}>
      <div className={dashboardPageAddrOverview__inferenceAddr}>
        <p>Inference addr: {inferenceAddr}</p>
      </div>
      <div className={dashboardPageAddrOverview__genericAddr}>
        <p>Management addr: {managementAddr}</p>
      </div>
      {statsdAddr && (
        <div className={dashboardPageAddrOverview__genericAddr}>
          <p>StatsD addr: {statsdAddr}</p>
          <p>StatsD prefix: {statsdPrefix}</p>
          <p>
            StatsD reporting interval: {statsdReportingIntervalMillis / 1000}s
          </p>
        </div>
      )}
    </div>
  );
}
