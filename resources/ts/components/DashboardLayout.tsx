import React, { ReactNode } from "react";
import { Link } from "wouter";

import {
  dashboard,
  dashboard__content,
  dashboard__header,
  dashboard__header__link,
  dashboard__header__linkActive,
  dashboard__header__track,
} from "./DashboardLayout.module.css";

function activeClassName(isActive: boolean) {
  return isActive
    ? `${dashboard__header__link} ${dashboard__header__linkActive}`
    : dashboard__header__link;
}

export function DashboardLayout({ children }: { children: ReactNode }) {
  return (
    <div className={dashboard}>
      <div className={dashboard__header}>
        <div className={dashboard__header__track}>
          <Link className={dashboard__header__link} href="/">
            Paddler 🏓
          </Link>
        </div>
        <div className={dashboard__header__track}>
          <Link className={activeClassName} href="/">
            Dashboard
          </Link>
          <Link className={activeClassName} href="/chat">
            Chat
          </Link>
        </div>
      </div>
      <div className={dashboard__content}>{children}</div>
    </div>
  );
}
