import React, { useCallback, useState, type MouseEvent } from "react";

import { ModalWindow } from "./ModalWindow";
import { ModelMetadataLoader } from "./ModelMetadataLoader";

import { agentListModelDetailsButton } from "./AgentListModelDetailsButton.module.css";

function displayLastPathPart(path: string | null | undefined): string {
  if (!path) {
    return "";
  }

  const parts = path.split("/");
  const last = parts.pop();

  if (!last) {
    return "";
  }

  return last;
}

export function AgentListModelDetailsButton({
  agentId,
  managementAddr,
  modelPath,
}: {
  agentId: string;
  managementAddr: string;
  modelPath: string;
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
      <abbr title={modelPath}>
        <button className={agentListModelDetailsButton} onClick={onClick}>
          {displayLastPathPart(modelPath)}
        </button>
      </abbr>
      {isDetailsVisible && (
        <ModalWindow onClose={onClose} title="Model Details">
          <ModelMetadataLoader
            agentId={agentId}
            managementAddr={managementAddr}
            modelPath={modelPath}
          />
        </ModalWindow>
      )}
    </>
  );
}
