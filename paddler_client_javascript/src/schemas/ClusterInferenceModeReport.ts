import { z } from "zod";

import { InferenceModeSchema } from "./InferenceMode";

export const ClusterInferenceModeReportSchema = z
  .object({
    balancer_instance: z.number().int(),
    inference_mode: InferenceModeSchema,
    version: z.number().int(),
  })
  .strict();

export type ClusterInferenceModeReport = z.infer<
  typeof ClusterInferenceModeReportSchema
>;
