import clsx from "clsx";
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
  return clsx(dashboard__header__link, {
    [dashboard__header__linkActive]: isActive,
  });
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
          <Link className={activeClassName} href="/prompt">
            Prompt
          </Link>
        </div>
      </div>
      <div className={dashboard__content}>{children}</div>
    </div>
  );
}
