import { z } from "zod";

import { StatusUpdateSchema } from "./StatusUpdate";

export const AgentSchema = z
  .object({
    agent_id: z.string(),
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
  .strict()
  .transform(function (agent) {
    return {
      data: agent,
      meta: {
        has_issues:
          agent.status.error ||
          true !== agent.status.is_authorized ||
          true === agent.status.is_connect_error ||
          true === agent.status.is_request_error ||
          true === agent.status.is_decode_error ||
          true === agent.status.is_deserialize_error ||
          true === agent.status.is_unexpected_response_status ||
          true !== agent.status.is_slots_endpoint_enabled ||
          agent.quarantined_until,
      },
    };
  });

export type Agent = z.infer<typeof AgentSchema>;
