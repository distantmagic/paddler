import { z } from "zod";

import { AgentDesiredModelSchema } from "./AgentDesiredModel";
import { ChatTemplateSchema } from "./ChatTemplate";
import { InferenceParametersSchema } from "./InferenceParameters";

export const AgentDesiredStateSchema = z
  .object({
    inference_parameters: InferenceParametersSchema,
    model: AgentDesiredModelSchema,
    override_chat_template: ChatTemplateSchema.nullable(),
  })
  .strict();

export type AgentDesiredState = z.infer<typeof AgentDesiredStateSchema>;
