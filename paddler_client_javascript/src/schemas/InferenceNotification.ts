import { z } from "zod";

import { ClusterInferenceModeReportSchema } from "./ClusterInferenceModeReport";

export const InferenceNotificationSchema = z
  .object({
    Notification: z
      .object({
        ClusterInferenceMode: ClusterInferenceModeReportSchema,
      })
      .strict(),
  })
  .strict();

export type InferenceNotification = z.infer<
  typeof InferenceNotificationSchema
>["Notification"];
