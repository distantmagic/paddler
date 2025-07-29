import { useEffect, useState } from "react";
import { z } from "zod";

export type EmptyState = {
  empty: true;
  error: null;
  loading: false;
  ok: false;
  response: null;
};

export type ErrorState = {
  empty: false;
  error: string;
  loading: false;
  response: null;
  ok: false;
};

export type LoadingState = {
  empty: false;
  error: null;
  loading: true;
  response: null;
  ok: false;
};

export type SuccessState<TResult> = {
  empty: false;
  error: null;
  loading: false;
  response: TResult;
  ok: true;
};

export type FetchJsonState<TResult> =
  | EmptyState
  | ErrorState
  | LoadingState
  | SuccessState<TResult>;

const emptyState: EmptyState = Object.freeze({
  empty: true,
  error: null,
  loading: false,
  response: null,
  ok: false,
});

const loadingState: LoadingState = Object.freeze({
  empty: false,
  error: null,
  loading: true,
  response: null,
  ok: false,
});

export function useFetchJson<TResponseSchema extends z.ZodType>({
  produceFetchPromise,
  responseSchema,
}: {
  produceFetchPromise(
    this: void,
    abortSignal: AbortSignal,
  ): null | Promise<Response>;
  responseSchema: TResponseSchema;
}): FetchJsonState<z.infer<TResponseSchema>> {
  const [fetchState, setFetchState] =
    useState<FetchJsonState<z.infer<TResponseSchema>>>(loadingState);

  useEffect(
    function () {
      const abortController = new AbortController();
      const fetchPromise = produceFetchPromise(abortController.signal);

      if (!fetchPromise) {
        setFetchState(emptyState);

        return function () {
          abortController.abort("Fetch promise was not provided.");
        };
      }

      setFetchState(loadingState);

      fetchPromise
        .then(function (response) {
          if (!response.ok) {
            throw new Error(`HTTP error status: ${response.status}`);
          }

          return response.json();
        })
        .then(function (result: unknown) {
          return responseSchema.parse(result);
        })
        .then(function (result: z.infer<TResponseSchema>) {
          setFetchState({
            empty: false,
            error: null,
            loading: false,
            response: result,
            ok: true,
          });
        })
        .catch(function (error: unknown) {
          setFetchState({
            empty: false,
            error: String(error),
            loading: false,
            response: null,
            ok: false,
          });
        });

      return function () {
        abortController.abort("Component unmounted or fetch cancelled.");
      };
    },
    [produceFetchPromise, responseSchema, setFetchState],
  );

  return fetchState;
}
