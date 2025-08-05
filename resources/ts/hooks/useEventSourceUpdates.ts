import { useEffect, useState } from "react";
import { z } from "zod";

export type ConnectedState = {
  data: undefined;
  isConnected: true;
  isConnectionError: false;
  isDeserializationError: false;
  isInitial: false;
  isOk: false;
};

export type ConnectionErrorState = {
  data: undefined;
  isConnected: false;
  isConnectionError: true;
  isDeserializationError: false;
  isInitial: false;
  isOk: false;
};

export type DataSnapshotState<TSchema extends z.ZodType> = {
  data: z.infer<TSchema>;
  isConnected: true;
  isConnectionError: false;
  isDeserializationError: false;
  isInitial: false;
  isOk: true;
};

export type DeserializationErrorState = {
  data: undefined;
  isConnected: true;
  isConnectionError: false;
  isDeserializationError: true;
  isInitial: false;
  isOk: false;
};

export type InitialStreamState = {
  data: undefined;
  isConnected: false;
  isConnectionError: false;
  isDeserializationError: false;
  isInitial: true;
  isOk: false;
};

export type StreamState<TSchema extends z.ZodType> =
  | ConnectedState
  | ConnectionErrorState
  | DataSnapshotState<TSchema>
  | DeserializationErrorState
  | InitialStreamState;

const connectedState: ConnectedState = Object.freeze({
  data: undefined,
  isConnected: true,
  isConnectionError: false,
  isDeserializationError: false,
  isInitial: false,
  isOk: false,
});

const connectionErrorState: ConnectionErrorState = Object.freeze({
  data: undefined,
  isConnected: false,
  isConnectionError: true,
  isDeserializationError: false,
  isInitial: false,
  isOk: false,
});

const deserializationErrorState: DeserializationErrorState = Object.freeze({
  data: undefined,
  isConnected: true,
  isConnectionError: false,
  isDeserializationError: true,
  isInitial: false,
  isOk: false,
});

const defaultStreamState: InitialStreamState = Object.freeze({
  data: undefined,
  isConnected: false,
  isConnectionError: false,
  isDeserializationError: false,
  isInitial: true,
  isOk: false,
});

export function useEventSourceUpdates<TSchema extends z.ZodType>({
  endpoint,
  schema,
}: {
  endpoint: string;
  schema: TSchema;
}): StreamState<TSchema> {
  const [streamState, setStreamState] =
    useState<StreamState<TSchema>>(defaultStreamState);

  useEffect(
    function () {
      const eventSource = new EventSource(endpoint);

      eventSource.addEventListener("error", function () {
        setStreamState(connectionErrorState);
      });

      eventSource.addEventListener("message", function (event) {
        if ("string" !== typeof event.data) {
          console.error("Received non-string data from SSE:", event.data);
          setStreamState(deserializationErrorState);

          return;
        }

        const parsed = JSON.parse(event.data);
        const result = schema.safeParse(parsed);

        if (!result.success) {
          console.error(
            "Deserialization error:",
            JSON.stringify(parsed, null, "  "),
            result.error.issues,
          );
          setStreamState(deserializationErrorState);
        } else {
          setStreamState({
            data: result.data,
            isConnected: true,
            isConnectionError: false,
            isDeserializationError: false,
            isInitial: false,
            isOk: true,
          });
        }
      });

      eventSource.addEventListener("open", function () {
        setStreamState(connectedState);
      });

      return function () {
        eventSource.close();
      };
    },
    [endpoint, schema, setStreamState],
  );

  return streamState;
}
