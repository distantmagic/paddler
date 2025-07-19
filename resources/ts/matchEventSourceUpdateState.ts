import { type ReactNode } from "react";
import { z } from "zod";
import {
  type ConnectedState,
  type ConnectionErrorState,
  type DataSnapshotState,
  type DeserializationErrorState,
  type InitialStreamState,
  type StreamState,
} from "./hooks/useEventSourceUpdates";

interface Handlers<TSchema extends z.ZodType> {
  connected(state: ConnectedState): ReactNode;
  connectionError(state: ConnectionErrorState): ReactNode;
  dataSnapshot(state: DataSnapshotState<TSchema>): ReactNode;
  deserializationError(state: DeserializationErrorState): ReactNode;
  initial(state: InitialStreamState): ReactNode;
}

export function matchEventSourceUpdateState<TSchema extends z.ZodType>(
  streamState: StreamState<TSchema>,
  handlers: Handlers<NoInfer<TSchema>>,
): ReactNode {
  if (streamState.isInitial) {
    return handlers.initial(streamState);
  }

  if (streamState.isConnectionError) {
    return handlers.connectionError(streamState);
  }

  if (streamState.isDeserializationError) {
    return handlers.deserializationError(streamState);
  }

  if (streamState.isOk) {
    return handlers.dataSnapshot(streamState);
  }

  return handlers.connected(streamState);
}
