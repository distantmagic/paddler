import { z } from "zod";

export const AgentSchema = z.object({
  agent_id: z.string(),
  agent_name: z.string().nullable(),
  error: z.string().nullable(),
  external_llamacpp_addr: z.string(),
  is_authorized: z.boolean().nullable(),
  is_slots_endpoint_enabled: z.boolean().nullable(),
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
  slots_idle: z.number(),
  slots_processing: z.number(),
});

export type Agent = z.infer<typeof AgentSchema>;
