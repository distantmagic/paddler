import { z } from "zod";

export const InferenceServiceGenerateTokensResponseSchema = z
  .object({
    Response: z.object({
      StreamChunk: z.object({
        request_id: z.string(),
        chunk: z.object({
          GeneratedToken: z.object({
            token: z.string(),
          }),
        }),
      }),
    }),
  })
  .strict()
  .transform(function (data) {
    return Object.freeze({
      requestId: data.Response.StreamChunk.request_id,
      token: data.Response.StreamChunk.chunk.GeneratedToken.token,
    });
  });

export type InferenceServiceGenerateTokensResponse = z.infer<
  typeof InferenceServiceGenerateTokensResponseSchema
>;
