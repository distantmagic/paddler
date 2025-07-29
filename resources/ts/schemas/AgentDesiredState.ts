import { z } from "zod";

import { AgentDesiredModelSchema } from "./AgentDesiredModel";
import { InferenceParametersSchema } from "./InferenceParameters";

export const AgentDesiredStateSchema = z
  .object({
    inference_parameters: InferenceParametersSchema,
    model: AgentDesiredModelSchema,
  })
  .strict();

export type AgentDesiredState = z.infer<typeof AgentDesiredStateSchema>;
