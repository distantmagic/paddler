import { z } from "zod";

export const AgentSchema = z
  .object({
    desired_slots_total: z.number(),
    id: z.string(),
    model_path: z.string().nullable(),
    name: z.string().nullable(),
    slots_processing: z.number(),
    slots_total: z.number(),
  })
  .strict();

export type Agent = z.infer<typeof AgentSchema>;
