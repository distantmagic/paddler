import { z } from "zod";

import { AgentSchema } from "./Agent";

export const UpstreamPeerPool = z
  .object({
    agents: z.array(AgentSchema),
    max_buffered_requests: z.number(),
    request_buffer_length: z.number(),
  })
  .strict();

export type AgentsResponse = z.infer<typeof UpstreamPeerPool>;
