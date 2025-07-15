import React, { type ReactNode } from "react";
import { createPortal } from "react-dom";

import {
  floatingStatus,
  floatingStatusOverlay,
} from "./FloatingStatus.module.css";

export function FloatingStatus({ children }: { children: ReactNode }) {
  return createPortal(
    <div className={floatingStatusOverlay}>
      <div className={floatingStatus}>{children}</div>
    </div>,
    document.body,
  );
}
