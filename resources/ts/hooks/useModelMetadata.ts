import { z } from "zod";

import { useFetchJson } from "./useFetchJson";

const responseSchema = z
  .object({
    metadata: z.record(z.string(), z.string()),
  })
  .strict()
  .nullable();

export function useModelMetadata({
  agentId,
  managementAddr,
}: {
  agentId: string;
  managementAddr: string;
}) {
  const result = useFetchJson({
    produceFetchPromise(signal) {
      return fetch(
        `//${managementAddr}/api/v1/agent/${agentId}/model_metadata`,
        {
          signal,
        },
      );
    },
    responseSchema,
  });

  return result;
}
