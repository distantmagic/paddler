import clsx from "clsx";
import React, {
  useCallback,
  useMemo,
  useState,
  type FormEvent,
  type InputEvent,
} from "react";

import { urlToAgentDesiredModel } from "../urlToAgentDesiredModel";
import {
  modelPage,
  modelPage__asideInfo,
  modelPage__details,
  modelPage__form,
  modelPage__formControls,
  modelPage__formLabel,
  modelPage__formLabel__title,
  modelPage__input,
  modelPage__main,
  modelPage__parameters,
  modelPage__payloadPreview,
  modelPage__payloadPreviewCorrect,
  modelPage__payloadPreviewError,
  modelPage__submitButton,
} from "./ModelPage.module.css";
import { ModelParameter } from "./ModelParameter";

export function ModelPage({ managementAddr }: { managementAddr: string }) {
  const [modelUriString, setModelUriString] = useState(
    "https://huggingface.co/Qwen/Qwen3-0.6B-GGUF/blob/main/Qwen3-0.6B-Q8_0.gguf",
  );

  const onModelUriInput = useCallback(
    function (evt: InputEvent<HTMLInputElement>) {
      setModelUriString(evt.currentTarget.value);
    },
    [setModelUriString],
  );

  const agentDesiredModel = useMemo(
    function () {
      if (!modelUriString) {
        return undefined;
      }

      try {
        const modelUri = new URL(modelUriString);

        return urlToAgentDesiredModel(modelUri);
      } catch (error) {
        return error;
      }
    },
    [modelUriString],
  );

  const agentDesiredModelError: null | string = useMemo(
    function () {
      if (agentDesiredModel instanceof Error) {
        return agentDesiredModel.message;
      }

      return null;
    },
    [agentDesiredModel],
  );

  const isAgentDesiredModelValid = useMemo(
    function () {
      return "string" !== typeof agentDesiredModelError;
    },
    [agentDesiredModelError],
  );

  const properPayload = useMemo(
    function () {
      if (!isAgentDesiredModelValid) {
        return null;
      }

      return JSON.stringify(
        {
          model: agentDesiredModel,
          model_parameters: {
            batch_n_tokens: 1024,
            context_size: 4096,
            min_p: 0.05,
            penalty_frequency: 0.0,
            penalty_last_n: 0,
            penalty_presence: 1.5,
            penalty_repeat: 1.0,
            temperature: 0.6,
            top_k: 40,
            top_p: 0.3,
          },
        },
        null,
        "  ",
      );
    },
    [agentDesiredModel, isAgentDesiredModelValid],
  );

  const payloadPreview = useMemo(
    function () {
      return agentDesiredModel instanceof Error
        ? agentDesiredModelError
        : properPayload;
    },
    [agentDesiredModel, agentDesiredModelError, properPayload],
  );

  const onSubmit = useCallback(
    function (evt: FormEvent<HTMLFormElement>) {
      evt.preventDefault();

      if ("string" !== typeof properPayload || !isAgentDesiredModelValid) {
        return;
      }

      fetch(`//${managementAddr}/api/v1/agent_desired_state`, {
        method: "PUT",
        headers: {
          "Content-Type": "application/json",
        },
        body: properPayload,
      })
        .then(function (response) {
          if (!response.ok) {
            throw new Error(
              `Failed to update agent desired state: ${response.statusText}`,
            );
          }
        })
        .catch(function (error: unknown) {
          console.error("Error updating agent desired state:", error);
        });
    },
    [isAgentDesiredModelValid, managementAddr, properPayload],
  );

  return (
    <div className={modelPage}>
      <aside className={modelPage__asideInfo}>
        <p>
          Paddler is based on <strong>llama.cpp</strong>, and it supports models
          in the <strong>GGUF</strong> format.
        </p>
        <p>Supported sources:</p>
        <dl>
          <dt>
            <a href="https://huggingface.co/" target="_blank">
              Hugging Face 🤗
            </a>
          </dt>
          <dd>
            <p>
              Each agent will individually download the model, and cache it
              locally.
            </p>
            <p>
              For example, you can use the following URL to download the Qwen-3
              0.6B model:
            </p>
            <code>
              https://huggingface.co/Qwen/Qwen3-0.6B-GGUF/blob/main/Qwen3-0.6B-Q8_0.gguf
            </code>
          </dd>
          <dt>Local File</dt>
          <dd>
            <p>File path is relative to the agent's working directory.</p>
            <p>
              If you want all the agents to use the same model, you need to
              ensure that the file is present in the same path on all agents.
            </p>
            <code>file:///path/to/your/model.gguf</code>
          </dd>
        </dl>
      </aside>
      <main className={modelPage__main}>
        <form className={modelPage__form} onSubmit={onSubmit}>
          <label className={modelPage__formLabel}>
            <div className={modelPage__formLabel__title}>Model URI</div>
            <input
              className={modelPage__input}
              onInput={onModelUriInput}
              placeholder="https://huggingface.co/..."
              required
              type="url"
              value={modelUriString}
            />
          </label>
          <fieldset className={modelPage__parameters}>
            <ModelParameter
              defaultvalue={512}
              description="Batch Tokens Size"
              name="batch_n_tokens"
            />
            <ModelParameter
              defaultvalue={4096}
              description="Context Size"
              name="context_size"
            />
            <ModelParameter
              defaultvalue={0.05}
              description="Minimum Token Probability"
              name="min_p"
            />
            <ModelParameter
              defaultvalue={0.0}
              description="Frequency Penalty"
              name="penalty_frequency"
            />
            <ModelParameter
              defaultvalue={-1}
              description="Number of last tokens to consider for penalty"
              name="penalty_last_n"
            />
            <ModelParameter
              defaultvalue={1.5}
              description="Penalty Presence"
              name="penalty_presence"
            />
            <ModelParameter
              defaultvalue={1.0}
              description="Penalty Repeat"
              name="penalty_repeat"
            />
            <ModelParameter
              defaultvalue={0.6}
              description="Temperature"
              name="temperature"
            />
            <ModelParameter
              defaultvalue={40}
              description="Top K"
              name="top_k"
            />
            <ModelParameter
              defaultvalue={0.3}
              description="Top P"
              name="top_p"
            />
          </fieldset>
          {undefined !== agentDesiredModel && (
            <details className={modelPage__details}>
              <summary>Payload Preview</summary>
              <pre
                className={clsx(modelPage__payloadPreview, {
                  [modelPage__payloadPreviewCorrect]: isAgentDesiredModelValid,
                  [modelPage__payloadPreviewError]: !isAgentDesiredModelValid,
                })}
              >
                {payloadPreview}
              </pre>
            </details>
          )}
          <div className={modelPage__formControls}>
            <button className={modelPage__submitButton}>Submit</button>
          </div>
        </form>
      </main>
    </div>
  );
}
