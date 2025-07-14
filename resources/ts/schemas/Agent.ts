import { z } from "zod";

export const AgentSchema = z
  .object({
    id: z.string(),
    name: z.string(),
    // last_update: z.object({
    //   nanos_since_epoch: z.number(),
    //   secs_since_epoch: z.number(),
    // }),
    // slots_taken: z.number(),
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
