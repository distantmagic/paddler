import { z } from "zod";

import { AgentSchema } from "./Agent";

export const AgentsResponseSchema = z
  .object({
    agents: z.array(AgentSchema),
  })
  .strict();

export type AgentsResponse = z.infer<typeof AgentsResponseSchema>;
