import clsx from "clsx";
import React, {
  useCallback,
  useMemo,
  useState,
  type FormEvent,
  type InputEvent,
} from "react";

import { urlToAgentDesiredState } from "../urlToAgentDesiredState";
import {
  modelPage,
  modelPage__asideInfo,
  modelPage__form,
  modelPage__formControls,
  modelPage__formLabel,
  modelPage__formLabel__title,
  modelPage__input,
  modelPage__main,
  modelPage__payloadPreview,
  modelPage__payloadPreviewCorrect,
  modelPage__payloadPreviewError,
  modelPage__submitButton,
} from "./ModelPage.module.css";

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

  const agentDesiredState = useMemo(
    function () {
      if (!modelUriString) {
        return undefined;
      }

      try {
        const modelUri = new URL(modelUriString);

        return urlToAgentDesiredState(modelUri);
      } catch (error) {
        return error;
      }
    },
    [modelUriString],
  );

  const onSubmit = useCallback(
    function (evt: FormEvent<HTMLFormElement>) {
      evt.preventDefault();

      if (agentDesiredState instanceof Error) {
        return;
      }

      console.log(agentDesiredState, managementAddr);
      fetch(`//${managementAddr}/api/v1/agent_desired_state`, {
        method: "PUT",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(agentDesiredState),
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
    [agentDesiredState, managementAddr],
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
          {undefined !== agentDesiredState && (
            <label className={modelPage__formLabel}>
              <div className={modelPage__formLabel__title}>Payload Preview</div>
              <pre
                className={clsx(modelPage__payloadPreview, {
                  [modelPage__payloadPreviewCorrect]: !(
                    agentDesiredState instanceof Error
                  ),
                  [modelPage__payloadPreviewError]:
                    agentDesiredState instanceof Error,
                })}
              >
                {agentDesiredState instanceof Error
                  ? agentDesiredState.message
                  : JSON.stringify(agentDesiredState, null, "  ")}
              </pre>
            </label>
          )}
          <div className={modelPage__formControls}>
            <button className={modelPage__submitButton}>Save</button>
          </div>
        </form>
      </main>
    </div>
  );
}
