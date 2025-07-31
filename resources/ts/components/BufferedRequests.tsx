import React from "react";

import { bufferedRequests } from "./BufferedRequests.module.css";

export function BufferedRequests({
  bufferedRequestTimeoutMillis,
  currentBufferedRequests,
  maxBufferedRequests,
}: {
  bufferedRequestTimeoutMillis: number;
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
      <p>
        Individual request's timeout: {bufferedRequestTimeoutMillis / 1000}s
      </p>
    </div>
  );
}
