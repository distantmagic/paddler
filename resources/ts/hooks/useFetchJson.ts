import { useEffect, useState } from "react";
import { z } from "zod";

export type ErrorState = {
  error: string;
  loading: false;
  response: null;
  ok: false;
};

export type LoadingState = {
  error: null;
  loading: true;
  response: null;
  ok: false;
};

export type SuccessState<TResult> = {
  error: null;
  loading: false;
  response: TResult;
  ok: true;
};

export type FetchJsonState<TResult> =
  | ErrorState
  | LoadingState
  | SuccessState<TResult>;

const defaultState: LoadingState = Object.freeze({
  error: null,
  loading: true,
  response: null,
  ok: false,
});

export function useFetchJson<TResponseSchema extends z.ZodType>({
  produceFetchPromise,
  responseSchema,
}: {
  produceFetchPromise(this: void, abortSignal: AbortSignal): Promise<Response>;
  responseSchema: TResponseSchema;
}): FetchJsonState<z.infer<TResponseSchema>> {
  const [fetchState, setFetchState] =
    useState<FetchJsonState<z.infer<TResponseSchema>>>(defaultState);

  useEffect(
    function () {
      const abortcontroller = new AbortController();

      produceFetchPromise(abortcontroller.signal)
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
            error: null,
            loading: false,
            response: result,
            ok: true,
          });
        })
        .catch(function (error: unknown) {
          setFetchState({
            error: String(error),
            loading: false,
            response: null,
            ok: false,
          });
        });

      return function () {
        abortcontroller.abort();
      };
    },
    [produceFetchPromise, responseSchema, setFetchState],
  );

  return fetchState;
}
