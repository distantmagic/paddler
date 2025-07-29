import { z } from "zod";

export const InferenceParametersSchema = z
  .object({
    batch_n_tokens: z.number(),
    context_size: z.number(),
    min_p: z.number(),
    penalty_frequency: z.number(),
    penalty_last_n: z.number(),
    penalty_presence: z.number(),
    penalty_repeat: z.number(),
    temperature: z.number(),
    top_k: z.number(),
    top_p: z.number(),
  })
  .strict();

export type InferenceParameters = z.infer<typeof InferenceParametersSchema>;
