import { z } from "zod";

import { HuggingFaceModelReferenceSchema } from "./HuggingFaceModelReference";

export const AgentDesiredModelSchema = z.union([
  z.object({
    HuggingFace: HuggingFaceModelReferenceSchema,
  }),
  z.object({
    Local: z.string(),
  }),
  z.literal("None"),
]);

export type AgentDesiredModel = z.infer<typeof AgentDesiredModelSchema>;
