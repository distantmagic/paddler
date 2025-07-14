import { z } from "zod";

export const AgentSchema = z
  .object({
    id: z.string(),
    name: z.string(),
    slots_processing: z.number(),
    slots_total: z.number(),
    // last_update: z.object({
    //   nanos_since_epoch: z.number(),
    //   secs_since_epoch: z.number(),
    // }),
  })
  .strict()
  .transform(function (agent) {
    return {
      data: agent,
      meta: {
        has_issues: false,
      },
    };
  });

export type Agent = z.infer<typeof AgentSchema>;
