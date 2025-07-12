import { z } from "zod";

export const StatusUpdateSchema = z
  .object({
    agent_name: z.string().nullable(),
    error: z.string().nullable(),
    external_llamacpp_addr: z.string(),
    is_authorized: z.boolean().nullable(),
    is_connect_error: z.boolean().nullable(),
    is_decode_error: z.boolean().nullable(),
    is_deserialize_error: z.boolean().nullable(),
    is_request_error: z.boolean().nullable(),
    is_slots_endpoint_enabled: z.boolean().nullable(),
    is_unexpected_response_status: z.boolean().nullable(),
    slots_idle: z.number(),
    slots_processing: z.number(),
    model: z.string().nullable(),
  })
  .strict();

export type StatusUpdate = z.infer<typeof StatusUpdateSchema>;
