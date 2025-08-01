import { z } from "zod";

export const ChatTemplateSchema = z
  .object({
    content: z.string(),
    id: z.string(),
    name: z.string(),
  })
  .strict();

export type ChatTemplate = z.infer<typeof ChatTemplateSchema>;
