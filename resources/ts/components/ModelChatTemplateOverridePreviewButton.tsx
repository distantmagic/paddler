import React, { useCallback, type MouseEvent } from "react";

import { modelChatTemplateOverridePreviewButton } from "./ModelChatTemplateOverridePreviewButton.module.css";

export function ModelChatTemplateOverridePreviewButton() {
  const onClick = useCallback(function (evt: MouseEvent<HTMLButtonElement>) {
    evt.preventDefault();
  }, []);

  return (
    <button
      className={modelChatTemplateOverridePreviewButton}
      onClick={onClick}
    >
      Custom chat template
    </button>
  );
}
