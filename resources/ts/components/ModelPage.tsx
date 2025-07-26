import React, {
  useCallback,
  useContext,
  useMemo,
  useState,
  type FormEvent,
  type InputEvent,
} from "react";
import { useLocation } from "wouter";

import { ModelParametersContext } from "../contexts/ModelParametersContext";
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
  modelPage__submitButton,
} from "./ModelPage.module.css";
import { ModelParameter } from "./ModelParameter";

export function ModelPage({ managementAddr }: { managementAddr: string }) {
  const { parameters } = useContext(ModelParametersContext);
  const [, navigate] = useLocation();
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

      return JSON.stringify({
        model: agentDesiredModel,
        model_parameters: parameters,
      });
    },
    [agentDesiredModel, isAgentDesiredModelValid, parameters],
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
          if (response.ok) {
            navigate("/");
          } else {
            throw new Error(
              `Failed to update agent desired state: ${response.statusText}`,
            );
          }
        })
        .catch(function (error: unknown) {
          console.error("Error updating agent desired state:", error);
        });
    },
    [isAgentDesiredModelValid, managementAddr, navigate, properPayload],
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
            <legend>Model Parameters</legend>
            <details className={modelPage__details}>
              <summary>What are these parameters?</summary>
              <p>
                These parameters control how the model behaves during inference.
                They can affect the quality, speed, and memory usage of the
                model.
              </p>
              <p>
                They are model-specific and are usually provided by the model
                authors.
              </p>
              <p>
                Experimenting with these settings is worth exploring to optimize
                performance for your specific needs. The main constraints you'll
                encounter are memory limits or thermal throttling during
                extended runs without adequate cooling. But honestly, nothing to
                lose sleep over - if your system handles intensive workloads
                like rendering, gaming marathons, or data processing, it'll
                handle LLMs just fine. 🙂
              </p>
            </details>
            <ModelParameter
              description="Batch Size (higher = more memory usage, lower = less inference speed)"
              name="batch_n_tokens"
            />
            <ModelParameter
              description="Context Size (higher = longer chat history, lower = less memory usage)"
              name="context_size"
            />
            <ModelParameter
              description="Minimum token probability to consider for selection"
              name="min_p"
            />
            <ModelParameter
              description="Frequency Penalty"
              name="penalty_frequency"
            />
            <ModelParameter
              description="Number of last tokens to consider for penalty (-1 = entire context, 0 = disabled)"
              name="penalty_last_n"
            />
            <ModelParameter
              description="Presence Penalty"
              name="penalty_presence"
            />
            <ModelParameter
              description="Repeated Token Penalty"
              name="penalty_repeat"
            />
            <ModelParameter description="Temperature" name="temperature" />
            <ModelParameter
              description="Number of tokens to consider for selection"
              name="top_k"
            />
            <ModelParameter
              description="Probability threshold for selecting tokens"
              name="top_p"
            />
          </fieldset>
          <div className={modelPage__formControls}>
            <button className={modelPage__submitButton}>Submit</button>
          </div>
        </form>
      </main>
    </div>
  );
}
