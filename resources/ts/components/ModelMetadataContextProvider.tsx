import React, { useMemo, useState, type ReactNode } from "react";

import {
  ModelMetadataContext,
  type FocusedMetadataParameter,
  type ModelMetadataContextValue,
} from "../contexts/ModelMetadataContext";

export function ModelMetadataContextProvider({
  children,
  metadata,
}: {
  children: ReactNode;
  metadata: Record<string, string>;
}) {
  const [focusedMetadataParameter, setFocusedMetadataParameter] = useState<
    undefined | FocusedMetadataParameter
  >();

  const value = useMemo<ModelMetadataContextValue>(
    function () {
      return Object.freeze({
        focusedMetadataParameter,
        metadata,
        setFocusedMetadataParameter,
      });
    },
    [focusedMetadataParameter, metadata, setFocusedMetadataParameter],
  );

  return (
    <ModelMetadataContext.Provider value={value}>
      {children}
    </ModelMetadataContext.Provider>
  );
}
