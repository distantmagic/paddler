import { createContext } from "react";

export type FocusedMetadataParameter = {
  metadataKey: string;
  metadataValue: string;
};

export type ModelMetadataContextValue = {
  focusedMetadataParameter: undefined | FocusedMetadataParameter;
  metadata: Record<string, string>;
  setFocusedMetadataParameter(
    this: void,
    metadataParameter: undefined | FocusedMetadataParameter,
  ): void;
};

export const ModelMetadataContext = createContext<ModelMetadataContextValue>({
  get focusedMetadataParameter(): never {
    throw new Error("ModelMetadataContext not provided");
  },
  get metadata(): never {
    throw new Error("ModelMetadataContext not provided");
  },
  setFocusedMetadataParameter() {
    throw new Error("ModelMetadataContext not provided");
  },
});
