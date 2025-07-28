import { useEffect, useState } from "react";

type ModelMetadataResult =
  | {
      error: null;
      loading: false;
      metadata: null | Record<string, string>;
      ok: true;
    }
  | {
      error: string;
      loading: false;
      metadata: null;
      ok: false;
    }
  | {
      error: null;
      loading: true;
      metadata: null;
      ok: false;
    };

const defaultModelMetadataResult: ModelMetadataResult = Object.freeze({
  error: null,
  loading: true,
  metadata: null,
  ok: false,
});

export function useModelMetadata({
  agentId,
  managementAddr,
}: {
  agentId: string;
  managementAddr: string;
}) {
  const [modelMetadataResult, setModelMetadataResult] =
    useState<ModelMetadataResult>(defaultModelMetadataResult);

  useEffect(
    function () {
      const abortController = new AbortController();

      fetch(`//${managementAddr}/api/v1/agent/${agentId}/model_metadata`, {
        signal: abortController.signal,
      })
        .then(function (response) {
          if (!response.ok) {
            throw new Error(
              `Failed to fetch model metadata: ${response.statusText}`,
            );
          }

          return response.json();
        })
        .then(function (metadata: null | { metadata: Record<string, string> }) {
          setModelMetadataResult({
            error: null,
            loading: false,
            metadata: metadata?.metadata ?? null,
            ok: true,
          });
        })
        .catch(function (error: unknown) {
          setModelMetadataResult({
            error: error instanceof Error ? error.message : String(error),
            loading: false,
            metadata: null,
            ok: false,
          });
        });

      return function () {
        abortController.abort();
      };
    },
    [agentId, managementAddr, setModelMetadataResult],
  );

  return {
    result: modelMetadataResult,
  };
}
