import { useCallback } from "react";
import { z } from "zod";

import { ChatTemplateSchema } from "../schemas/ChatTemplate";
import { useFetchJson } from "./useFetchJson";

const responseSchema = z
  .object({
    chat_template: ChatTemplateSchema,
  })
  .strict();

export function useChatTemplate({
  chatTemplateId,
  managementAddr,
}: {
  chatTemplateId: string;
  managementAddr: string;
}) {
  const produceFetchPromise = useCallback(
    function (signal: AbortSignal) {
      return fetch(
        `//${managementAddr}/api/v1/chat_template/${chatTemplateId}`,
        {
          signal,
        },
      );
    },
    [chatTemplateId, managementAddr],
  );

  const result = useFetchJson({
    produceFetchPromise,
    responseSchema,
  });

  return result;
}
