import React from "react";

import { bufferedRequests } from "./BufferedRequests.module.css";

export function BufferedRequests({
  bufferedRequestTimeoutMilis,
  currentBufferedRequests,
  maxBufferedRequests,
}: {
  bufferedRequestTimeoutMilis: number;
  currentBufferedRequests: number;
  maxBufferedRequests: number;
}) {
  return (
    <div className={bufferedRequests}>
      <p>Buffered requests</p>
      <progress value={currentBufferedRequests} max={maxBufferedRequests} />
      <p>
        {currentBufferedRequests}/{maxBufferedRequests}
      </p>
      <p>Individual request's timeout: {bufferedRequestTimeoutMilis} ms</p>
    </div>
  );
}
