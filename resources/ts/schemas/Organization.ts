import { z } from "zod";

export const OrganizationSchema = z
  .object({
    data: z
      .object({
        V1: z
          .object({
            id: z.string(),
            name: z.string(),
          })
          .strict(),
      })
      .strict(),
    version: z.string(),
  })
  .strict()
  .transform(function ({ data }) {
    return {
      id: data.V1.id,
      name: data.V1.name,
    };
  });

export type Organization = z.infer<typeof OrganizationSchema>;
