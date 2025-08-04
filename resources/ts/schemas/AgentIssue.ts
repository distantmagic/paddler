import { z } from "zod";

export const AgentIssueSchema = z.union([
  z.object({
    ChatTemplateDoesNotCompile: z.object({
      error: z.string(),
      template_content: z.string(),
    }),
  }),
  z.object({
    HuggingFaceCannotAcquireLock: z.string(),
  }),
  z.object({
    HuggingFaceModelDoesNotExist: z.string(),
  }),
  z.object({
    ModelCannotBeLoaded: z.string(),
  }),
  z.object({
    ModelFileDoesNotExist: z.string(),
  }),
  z.object({
    SlotCannotStart: z.object({
      error: z.string(),
      slot_index: z.number(),
    }),
  }),
  z.object({
    UnableToFindChatTemplate: z.string(),
  }),
]);

export type AgentIssue = z.infer<typeof AgentIssueSchema>;
