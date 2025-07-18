import React from "react";

import {
  modelPage,
  modelPage__asideInfo,
  modelPage__form,
  modelPage__formLabel,
  modelPage__formLabel__title,
  modelPage__input,
  modelPage__main,
} from "./ModelPage.module.css";

export function ModelPage() {
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
        <form className={modelPage__form}>
          <label className={modelPage__formLabel}>
            <div className={modelPage__formLabel__title}>Model URI</div>
            <input
              className={modelPage__input}
              placeholder="https://huggingface.co/..."
              type="url"
            />
          </label>
        </form>
      </main>
    </div>
  );
}
