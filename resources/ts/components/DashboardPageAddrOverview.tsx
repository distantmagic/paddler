import React from "react";

import {
  dashboardPageAddrOverview,
  dashboardPageAddrOverview__inferenceAddr,
  dashboardPageAddrOverview__managementAddr,
} from "./DashboardPageAddrOverview.module.css";

export function DashboardPageAddrOverview({
  inferenceAddr,
  managementAddr,
}: {
  inferenceAddr: string;
  managementAddr: string;
}) {
  return (
    <div className={dashboardPageAddrOverview}>
      <div className={dashboardPageAddrOverview__inferenceAddr}>
        Inference addr: {inferenceAddr}
      </div>
      <div className={dashboardPageAddrOverview__managementAddr}>
        Management addr: {managementAddr}
      </div>
    </div>
  );
}
