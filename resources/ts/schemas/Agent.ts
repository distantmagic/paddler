import { z } from "zod";

import { AgentIssueSchema } from "./AgentIssue";

export const AgentSchema = z
  .object({
    desired_slots_total: z.number(),
    id: z.string(),
    issues: z.array(AgentIssueSchema),
    model_path: z.string().nullable(),
    name: z.string().nullable(),
    slots_processing: z.number(),
    slots_total: z.number(),
  })
  .strict();

export type Agent = z.infer<typeof AgentSchema>;
