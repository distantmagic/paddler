import React, { useContext } from "react";

import { ModelMetadataContext } from "../contexts/ModelMetadataContext";
import { ModalWindow } from "./ModalWindow";
import { ModelChatTemplatePreviewButton } from "./ModelChatTemplatePreviewButton";
import { ModelMetadataFocusedParameter } from "./ModelMetadataFocusedParameter";

import {
  modelMetadata,
  modelMetadata__parameter,
  modelMetadata__parameter__title,
  modelMetadata__parameter__value,
} from "./ModelMetadata.module.css";

export function ModelMetadata({
  agentName,
  onClose,
}: {
  agentName: null | string;
  onClose(this: void): void;
}) {
  const { focusedMetadataParameter, metadata } =
    useContext(ModelMetadataContext);

  return (
    <ModalWindow
      onClose={onClose}
      title={`${agentName} / Metadata${focusedMetadataParameter ? ` / ${focusedMetadataParameter.metadataKey}` : ""}`}
    >
      {focusedMetadataParameter ? (
        <ModelMetadataFocusedParameter
          focusedMetadataParameter={focusedMetadataParameter}
        />
      ) : (
        <div className={modelMetadata}>
          {Object.entries(metadata).map(function ([
            metadataKey,
            metadataValue,
          ]) {
            return (
              <div className={modelMetadata__parameter} key={metadataKey}>
                <div className={modelMetadata__parameter__title}>
                  {metadataKey}:
                </div>
                <div className={modelMetadata__parameter__value}>
                  {"tokenizer.chat_template" === metadataKey ? (
                    <ModelChatTemplatePreviewButton metadataKey={metadataKey} />
                  ) : (
                    metadataValue
                  )}
                </div>
              </div>
            );
          })}
        </div>
      )}
    </ModalWindow>
  );
}
