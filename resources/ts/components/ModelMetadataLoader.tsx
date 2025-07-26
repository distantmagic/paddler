import React, { useEffect, useState } from "react";

import iconHourglass from "../../icons/hourglass.svg";
import {
  modelMetadataLoader,
  modelMetadataLoader__spinner,
} from "./ModelMetadataLoader.module.css";

type ModelMetadataResult =
  | {
      error: null;
      loading: false;
      metadata: Record<string, string>;
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

export function ModelMetadataLoader({
  agentId,
  managementAddr,
  modelPath,
}: {
  agentId: string;
  managementAddr: string;
  modelPath: string;
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
        .then(function ({ metadata }: { metadata: Record<string, string> }) {
          setModelMetadataResult({
            error: null,
            loading: false,
            metadata,
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

  return (
    <div className={modelMetadataLoader}>
      <dl>
        <dt>Model Path:</dt>
        <dd>{modelPath}</dd>
      </dl>
      {modelMetadataResult.loading && (
        <div className={modelMetadataLoader__spinner}>
          <img src={iconHourglass} alt="Loading..." />
          <span>Loading model metadata...</span>
        </div>
      )}
      {modelMetadataResult.ok && (
        <dl>
          {Object.entries(modelMetadataResult.metadata).map(function ([
            key,
            value,
          ]) {
            return (
              <React.Fragment key={key}>
                <dt>{key}:</dt>
                <dd>{value}</dd>
              </React.Fragment>
            );
          })}
        </dl>
      )}
    </div>
  );
}
