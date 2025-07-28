import React from "react";

import { useModelMetadata } from "../hooks/useModelMetadata";
import { ModalWindow } from "./ModalWindow";
import { ModelMetadata } from "./ModelMetadata";
import { ModelMetadataContextProvider } from "./ModelMetadataContextProvider";

import iconHourglass from "../../icons/hourglass.svg";
import {
  modelMetadataLoader,
  modelMetadataLoader__spinner,
} from "./ModelMetadataLoader.module.css";

export function ModelMetadataLoader({
  agentId,
  agentName,
  managementAddr,
  onClose,
}: {
  agentId: string;
  agentName: null | string;
  managementAddr: string;
  onClose(this: void): void;
}) {
  const {
    result: { error, loading, metadata, ok },
  } = useModelMetadata({
    agentId,
    managementAddr,
  });

  return (
    <div className={modelMetadataLoader}>
      {loading && (
        <ModalWindow onClose={onClose} title={`${agentName} / Loading`}>
          <div className={modelMetadataLoader__spinner}>
            <img src={iconHourglass} alt="Loading..." />
            <span>Loading model metadata...</span>
          </div>
        </ModalWindow>
      )}
      {error && (
        <ModalWindow onClose={onClose} title={`${agentName} / Error`}>
          <span>Error: {error}</span>
        </ModalWindow>
      )}
      {ok && !metadata && (
        <ModalWindow onClose={onClose} title={`${agentName}`}>
          <span>No model loaded</span>
        </ModalWindow>
      )}
      {metadata && (
        <ModelMetadataContextProvider metadata={metadata}>
          <ModelMetadata agentName={agentName} onClose={onClose} />
        </ModelMetadataContextProvider>
      )}
    </div>
  );
}
