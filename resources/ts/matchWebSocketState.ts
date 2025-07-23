import { type ReactNode } from "react";
import {
  type ConnectingState,
  type ConnectionClosedState,
  type ConnectionErrorState,
  type ConnectionOpenedState,
  type SocketState,
} from "./hooks/useWebSocket";

interface Handlers {
  connected(socketState: ConnectionOpenedState): ReactNode;
  connecting(socketState: ConnectingState): ReactNode;
  connectionClosed(socketState: ConnectionClosedState): ReactNode;
  connectionError(socketState: ConnectionErrorState): ReactNode;
}

export function matchWebSocketState(
  socketState: SocketState,
  handlers: Handlers,
): ReactNode {
  if (socketState.isConnected) {
    return handlers.connected(socketState);
  }

  if (socketState.isConnectionClosed) {
    return handlers.connectionClosed(socketState);
  }

  if (socketState.isConnectionError) {
    return handlers.connectionError(socketState);
  }

  return handlers.connecting(socketState);
}
