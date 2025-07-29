import React from "react";

import { notificationCount } from "./NotificationCount.module.css";

export function NotificationCount({ count }: { count: number }) {
  return <span className={notificationCount}>{count}</span>;
}
