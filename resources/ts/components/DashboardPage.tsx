import React from "react";

import { AgentListStream } from "./AgentListStream";
import { DashboardBufferedRequests } from "./DashboardBufferedRequests";

import {
  dashboardPage,
  dashboardPage__serviceBlock,
} from "./DashboardPage.module.css";

export function DashboardPage({
  inferenceAddr,
  managementAddr,
}: {
  inferenceAddr: string;
  managementAddr: string;
}) {
  return (
    <div className={dashboardPage}>
      <div className={dashboardPage__serviceBlock}>
        <p>Inference addr: {inferenceAddr}</p>
        <p>Management addr: {managementAddr}</p>
      </div>
      <div className={dashboardPage__serviceBlock}>
        <DashboardBufferedRequests />
      </div>
      <div className={dashboardPage__serviceBlock}>
        <AgentListStream managementAddr={managementAddr} />
      </div>
    </div>
  );
}
