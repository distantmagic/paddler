import { type ReactNode } from "react";

import {
  type EmptyState,
  type ErrorState,
  type FetchJsonState,
  type LoadingState,
  type SuccessState,
} from "./hooks/useFetchJson";

interface Handlers<TResponse> {
  empty(state: EmptyState): ReactNode;
  error(state: ErrorState): ReactNode;
  loading(state: LoadingState): ReactNode;
  ok(state: SuccessState<TResponse>): ReactNode;
}

export function matchFetchJsonState<TResponse>(
  state: FetchJsonState<TResponse>,
  handlers: Handlers<TResponse>,
): ReactNode {
  if (state.empty) {
    return handlers.empty(state);
  }

  if (state.loading) {
    return handlers.loading(state);
  }

  if (state.error) {
    return handlers.error(state);
  }

  if (state.ok) {
    return handlers.ok(state);
  }

  throw new Error(`Invalid state: ${JSON.stringify(state)}`);
}
