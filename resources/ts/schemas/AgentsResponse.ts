import { z } from "zod";

import { AgentSchema } from "./Agent";

export const UpstreamPeerPool = z
  .object({
    agents: z.array(AgentSchema),
    request_buffer_length: z.number(),
  })
  .strict();

export type AgentsResponse = z.infer<typeof UpstreamPeerPool>;
