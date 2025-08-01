import { z } from "zod";

export const ChatTemplateHeadSchema = z
  .object({
    id: z.string(),
    name: z.string(),
  })
  .strict();

export type ChatTemplateHead = z.infer<typeof ChatTemplateHeadSchema>;
