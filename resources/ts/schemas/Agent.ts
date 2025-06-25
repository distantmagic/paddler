import { z } from "zod";

import { StatusUpdateSchema } from "./StatusUpdate";

export const AgentSchema = z
  .object({
    agent_id: z.string(),
    model: z.string(),
    last_update: z.object({
      nanos_since_epoch: z.number(),
      secs_since_epoch: z.number(),
    }),
    quarantined_until: z
      .object({
        nanos_since_epoch: z.number(),
        secs_since_epoch: z.number(),
      })
      .nullable(),
    slots_taken: z.number(),
    slots_taken_since_last_status_update: z.number(),
    status: StatusUpdateSchema,
  })
  .strict();

export type Agent = z.infer<typeof AgentSchema>;
