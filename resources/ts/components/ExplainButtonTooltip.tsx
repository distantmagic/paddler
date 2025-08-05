import React from "react";

import { usePrompt } from "../hooks/usePrompt";

export function ExplainButtonTooltip({
  inferenceAddr,
  prompt,
  systemPrompt,
}: {
  inferenceAddr: string;
  prompt: string;
  systemPrompt: string;
}) {
  const { message } = usePrompt({
    inferenceAddr,
    prompt,
    systemPrompt,
  });

  return <div>{message}</div>;
}
