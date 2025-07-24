import { useEffect, useRef, useState } from "react";

export type ConnectingState = {
  isConnected: false;
  isConnectionClosed: false;
  isConnectionError: false;
  webSocket: null;
};

export type ConnectionClosedState = {
  isConnected: false;
  isConnectionClosed: true;
  isConnectionError: false;
  webSocket: null;
};

export type ConnectionErrorState = {
  isConnected: false;
  isConnectionClosed: false;
  isConnectionError: true;
  webSocket: null;
};

export type ConnectionOpenedState = {
  isConnected: true;
  isConnectionClosed: false;
  isConnectionError: false;
  webSocket: WebSocket;
};

export type SocketState =
  | ConnectingState
  | ConnectionClosedState
  | ConnectionErrorState
  | ConnectionOpenedState;

const connectionClosedState: ConnectionClosedState = Object.freeze({
  isConnected: false,
  isConnectionClosed: true,
  isConnectionError: false,
  webSocket: null,
});

const connectionErrorState: ConnectionErrorState = Object.freeze({
  isConnected: false,
  isConnectionClosed: false,
  isConnectionError: true,
  webSocket: null,
});

const defaultSocketState: ConnectingState = Object.freeze({
  isConnected: false,
  isConnectionClosed: false,
  isConnectionError: false,
  webSocket: null,
});

const MAX_RECONNECT_DEBOUNCE_TIME_INCREASE = 3;
const RECONNECT_DELAY = 600;

function incrementVersion(version: number): number {
  return version + 1;
}

export function useWebSocket({ endpoint }: { endpoint: string }): SocketState {
  const [socketState, setSocketState] =
    useState<SocketState>(defaultSocketState);
  const [version, setVersion] = useState(0);
  const [webSocket, setWebSocket] = useState<null | WebSocket>(null);
  const reconnectAttempts = useRef(0);

  useEffect(
    function () {
      function connect() {
        const webSocket = new WebSocket(endpoint);

        setWebSocket(webSocket);
      }

      if (version < 1) {
        connect();

        return;
      }

      reconnectAttempts.current += 1;

      const timeoutId = setTimeout(
        connect,
        Math.min(
          reconnectAttempts.current,
          MAX_RECONNECT_DEBOUNCE_TIME_INCREASE,
        ) * RECONNECT_DELAY,
      );

      return function () {
        clearTimeout(timeoutId);
      };
    },
    [endpoint, setWebSocket, version],
  );

  useEffect(
    function () {
      if (!webSocket) {
        return;
      }

      return function () {
        webSocket.close();
      };
    },
    [webSocket],
  );

  useEffect(
    function () {
      if (!webSocket) {
        return;
      }

      webSocket.addEventListener("close", function () {
        setSocketState(connectionClosedState);
        setVersion(incrementVersion);
      });

      webSocket.addEventListener("error", function (event) {
        console.error("WebSocket error:", event);
        setSocketState(connectionErrorState);
        setVersion(incrementVersion);
      });

      webSocket.addEventListener("open", function () {
        reconnectAttempts.current = 0;

        setSocketState({
          isConnected: true,
          isConnectionClosed: false,
          isConnectionError: false,
          webSocket: webSocket,
        });
      });
    },
    [endpoint, setSocketState, setVersion, webSocket],
  );

  return socketState;
}
