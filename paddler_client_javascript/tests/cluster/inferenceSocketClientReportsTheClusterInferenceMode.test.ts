import { strictEqual } from "node:assert/strict";
import { once } from "node:events";
import { test } from "node:test";
import { firstValueFrom } from "rxjs";

import { fetchJson } from "../../src/fetchJson";
import { inferenceSocketClient } from "../../src/inferenceSocketClient";
import { BalancerDesiredStateSchema } from "../../src/schemas/BalancerDesiredState";
import { inferenceSocketUrl } from "../inferenceSocketUrl";
import { putBalancerDesiredState } from "../putBalancerDesiredState";
import { PADDLER_DEFAULT_BUFFERED_REQUEST_TIMEOUT_MILLISECONDS } from "../SpawnedBalancer";
import { withSpawnedBalancer } from "../withSpawnedBalancer";

test("inferenceSocketClient reports the cluster inference mode", async function () {
  await withSpawnedBalancer(
    {
      bufferedRequestTimeoutMilliseconds:
        PADDLER_DEFAULT_BUFFERED_REQUEST_TIMEOUT_MILLISECONDS,
    },
    async function (addresses) {
      const storedDesiredState = await fetchJson({
        schema: BalancerDesiredStateSchema,
        signal: new AbortController().signal,
        url: `http://${addresses.management}/api/v1/balancer_desired_state`,
      });

      await putBalancerDesiredState({
        desiredState: {
          ...storedDesiredState,
          inference_mode: "TextGenerationEmbeddingsAndDecisions",
        },
        managementAddress: addresses.management,
      });

      const webSocket = new WebSocket(inferenceSocketUrl(addresses));

      try {
        const { clusterInferenceMode$ } = inferenceSocketClient({ webSocket });
        const firstClusterInferenceMode = firstValueFrom(clusterInferenceMode$);

        await once(webSocket, "open");

        strictEqual(
          await firstClusterInferenceMode,
          "TextGenerationEmbeddingsAndDecisions",
        );
      } finally {
        webSocket.close();
      }
    },
  );
});
