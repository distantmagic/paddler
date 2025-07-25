import { z } from "zod";

export const InferenceServiceGenerateTokensResponseSchema = z
  .object({
    Response: z.object({
      request_id: z.string(),
      response: z.object({
        GeneratedToken: z.object({
          generated_token_result: z.union([
            z.literal("Done"),
            z.object({
              Token: z.object({
                token: z.string(),
              }),
            }),
          ]),
          slot: z.number(),
        }),
      }),
    }),
  })
  .strict()
  .transform(function (data):
    | {
        done: true;
        request_id: string;
        token: null;
      }
    | {
        done: false;
        request_id: string;
        token: string;
      } {
    if (
      data.Response.response.GeneratedToken.generated_token_result === "Done"
    ) {
      return Object.freeze({
        done: true,
        request_id: data.Response.request_id,
        token: null,
      });
    }

    return Object.freeze({
      done: false,
      request_id: data.Response.request_id,
      token:
        data.Response.response.GeneratedToken.generated_token_result.Token
          .token,
    });
  });

export type InferenceServiceGenerateTokensResponse = z.infer<
  typeof InferenceServiceGenerateTokensResponseSchema
>;
