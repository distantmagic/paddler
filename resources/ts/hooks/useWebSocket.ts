import { useEffect, useState } from "react";

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

const RECONNECT_DELAY = 300;

function incrementVersion(version: number): number {
  return version + 1;
}

export function useWebSocket({ endpoint }: { endpoint: string }): SocketState {
  const [socketState, setSocketState] =
    useState<SocketState>(defaultSocketState);
  const [version, setVersion] = useState(0);
  const [webSocket, setWebSocket] = useState<null | WebSocket>(null);

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

      const timeoutId = setTimeout(connect, RECONNECT_DELAY);

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

      webSocket.addEventListener("close", function (event) {
        console.log("WebSocket connection closed:", event);
        setSocketState(connectionClosedState);
        setVersion(incrementVersion);
      });

      webSocket.addEventListener("error", function (event) {
        console.error("WebSocket error:", event);
        setSocketState(connectionErrorState);
        setVersion(incrementVersion);
      });

      webSocket.addEventListener("open", function (event) {
        console.log("WebSocket connection opened:", event);
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
