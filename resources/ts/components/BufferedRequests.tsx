import React, { useContext } from "react";

import { PaddlerConfigurationContext } from "../contexts/PaddlerConfigurationContext";

import { bufferedRequests } from "./BufferedRequests.module.css";

export function BufferedRequests({
  currentBufferedRequests,
}: {
  currentBufferedRequests: number;
}) {
  const { bufferedRequestTimeoutMillis, maxBufferedRequests } = useContext(
    PaddlerConfigurationContext,
  );

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
