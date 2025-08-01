import { z } from "zod";

import { ChatTemplateHeadSchema } from "./ChatTemplateHead";

export const ChatTemplateHeadsResponseSchema = z
  .object({
    chat_template_heads: z.array(ChatTemplateHeadSchema),
  })
  .strict();

export type ChatTemplateHeads = z.infer<typeof ChatTemplateHeadsResponseSchema>;
