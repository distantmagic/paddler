import React, { ReactNode } from "react";

export function DashboardLayout({
  children,
  currentTick,
}: {
  children: ReactNode;
  currentTick: number;
}) {
  const degrees = (currentTick % 60) * 6;

  return (
    <div className="dashboard">
      <div className="dashboard__content">{children}</div>
      <div className="dashboard__status-bar">
        <div>current tick: {currentTick}</div>
        <div className="dashobard__status-bar__ticker">
          <svg viewBox="0 0 100 100">
            <circle
              cx="50"
              cy="50"
              r="45"
              fill="none"
              stroke="white"
              strokeWidth="2"
            />
            <line
              x1="50"
              y1="50"
              x2="50"
              y2="10"
              stroke="white"
              strokeWidth="2"
              transform={`rotate(${degrees}, 50, 50)`}
              style={{ transition: "transform 0.1s ease" }}
            />
          </svg>
        </div>
      </div>
    </div>
  );
}
