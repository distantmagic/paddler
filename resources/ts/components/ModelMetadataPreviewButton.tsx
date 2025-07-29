import React, { useCallback, useState, type MouseEvent } from "react";

import { ModelMetadataLoader } from "./ModelMetadataLoader";

import { modelMetadataPreviewButton } from "./ModelMetadataPreviewButton.module.css";

export function ModelMetadataPreviewButton({
  agentId,
  agentName,
  managementAddr,
}: {
  agentId: string;
  agentName: null | string;
  managementAddr: string;
}) {
  const [isDetailsVisible, setIsDetailsVisible] = useState(false);

  const onClick = useCallback(
    function (evt: MouseEvent<HTMLButtonElement>) {
      evt.preventDefault();

      setIsDetailsVisible(true);
    },
    [setIsDetailsVisible],
  );

  const onClose = useCallback(
    function () {
      setIsDetailsVisible(false);
    },
    [setIsDetailsVisible],
  );

  return (
    <>
      <button className={modelMetadataPreviewButton} onClick={onClick}>
        Metadata
      </button>
      {isDetailsVisible && (
        <ModelMetadataLoader
          agentId={agentId}
          agentName={agentName}
          managementAddr={managementAddr}
          onClose={onClose}
        />
      )}
    </>
  );
}
