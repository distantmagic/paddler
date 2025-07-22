import { z } from "zod";

export const InferenceServiceGenerateTokensResponseSchema = z
  .object({
    Response: z.object({
      request_id: z.string(),
      response: z.object({
        GeneratedToken: z.object({
          generated_token_result: z.object({
            Token: z.object({
              token: z.string(),
            }),
          }),
        }),
      }),
    }),
  })
  .strict()
  .transform(function (data) {
    return Object.freeze({
      requestId: data.Response.request_id,
      token:
        data.Response.response.GeneratedToken.generated_token_result.Token
          .token,
    });
  });

export type InferenceServiceGenerateTokensResponse = z.infer<
  typeof InferenceServiceGenerateTokensResponseSchema
>;
