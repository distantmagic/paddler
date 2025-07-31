import React from "react";

import { useEventSourceUpdates } from "../hooks/useEventSourceUpdates";
import { matchEventSourceUpdateState } from "../matchEventSourceUpdateState";
import { BufferedRequestsResponseSchema } from "../schemas/BufferedRequestsResponse";
import { BufferedRequests } from "./BufferedRequests";
import { FloatingStatus } from "./FloatingStatus";

export function BufferedRequestsStream({
  bufferedRequestTimeoutMilis,
  managementAddr,
  maxBufferedRequests,
}: {
  bufferedRequestTimeoutMilis: number;
  managementAddr: string;
  maxBufferedRequests: number;
}) {
  const eventSourceUpdateState = useEventSourceUpdates({
    schema: BufferedRequestsResponseSchema,
    endpoint: `//${managementAddr}/api/v1/buffered_requests/stream`,
  });

  return matchEventSourceUpdateState(eventSourceUpdateState, {
    connected() {
      return (
        <FloatingStatus>
          Connected to the server, waiting for buffered requests update...
        </FloatingStatus>
      );
    },
    connectionError() {
      return (
        <FloatingStatus>
          Cannot connect to the server. Will try again in a moment...
        </FloatingStatus>
      );
    },
    dataSnapshot({ data: { buffered_requests_current } }) {
      return (
        <BufferedRequests
          bufferedRequestTimeoutMilis={bufferedRequestTimeoutMilis}
          currentBufferedRequests={buffered_requests_current}
          maxBufferedRequests={maxBufferedRequests}
        />
      );
    },
    deserializationError() {
      return (
        <FloatingStatus>
          Error deserializing data from the server
        </FloatingStatus>
      );
    },
    initial() {
      return <FloatingStatus>Connecting to the server...</FloatingStatus>;
    },
  });
}
