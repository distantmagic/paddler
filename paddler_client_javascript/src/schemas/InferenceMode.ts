import { z } from "zod";

export const inferenceModes = [
  "TextGenerationAndEmbeddings",
  "TextGenerationEmbeddingsAndDecisions",
] as const;

export const InferenceModeSchema = z.enum(inferenceModes);

export type InferenceMode = z.infer<typeof InferenceModeSchema>;
