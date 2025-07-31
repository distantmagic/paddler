import React from "react";

import { dashboardBufferedRequests } from "./DashboardBufferedRequests.module.css";

export function DashboardBufferedRequests({
  bufferedRequestTimeoutMilis,
  maxBufferedRequests,
}: {
  bufferedRequestTimeoutMilis: number;
  maxBufferedRequests: number;
}) {
  return (
    <div className={dashboardBufferedRequests}>
      <p>Buffered requests</p>
      <progress value={0} max={maxBufferedRequests} />
      <p>0/{maxBufferedRequests}</p>
      <p>Individual request's timeout: {bufferedRequestTimeoutMilis} ms</p>
    </div>
  );
}
