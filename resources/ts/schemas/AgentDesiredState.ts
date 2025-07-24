import { z } from "zod";

import { HuggingFaceModelReferenceSchema } from "./HuggingFaceModelReference";

const AgentDesiredModelSchema = z.union([
  z.object({
    HuggingFace: HuggingFaceModelReferenceSchema,
  }),
  z.object({
    Local: z.string(),
  }),
]);

// eslint-disable-next-line @typescript-eslint/no-unused-vars
const AgentDesiredStateSchema = z.object({
  model: AgentDesiredModelSchema,
});

export type AgentDesiredState = z.infer<typeof AgentDesiredStateSchema>;
