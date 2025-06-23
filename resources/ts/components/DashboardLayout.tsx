import React, { ReactNode } from "react";

import { dashboard, dashboard__content } from "./DashboardLayout.module.css";

export function DashboardLayout({ children }: { children: ReactNode }) {
  return (
    <div className={dashboard}>
      <div className={dashboard__content}>{children}</div>
    </div>
  );
}
