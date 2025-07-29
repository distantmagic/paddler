import { z } from "zod";

export const AgentIssueSchema = z.union([
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
]);

export type AgentIssue = z.infer<typeof AgentIssueSchema>;
