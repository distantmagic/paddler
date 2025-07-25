import { z } from "zod";

import { HuggingFaceModelReferenceSchema } from "./HuggingFaceModelReference";

// eslint-disable-next-line @typescript-eslint/no-unused-vars
const AgentDesiredModelSchema = z.union([
  z.object({
    HuggingFace: HuggingFaceModelReferenceSchema,
  }),
  z.object({
    Local: z.string(),
  }),
]);

export type AgentDesiredModel = z.infer<typeof AgentDesiredModelSchema>;
