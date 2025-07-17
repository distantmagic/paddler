export function webSocketProtocol(windowProtocol: string): string {
  return windowProtocol === "https:" ? "wss:" : "ws:";
}
