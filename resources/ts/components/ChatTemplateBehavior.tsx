import React from "react";

import {
  chatTemplateBehavior,
  chatTemplateBehavior__description,
  chatTemplateBehavior__option,
  chatTemplateBehavior__optionList,
  chatTemplateBehavior__radio,
} from "./ChatTemplateBehavior.module.css";

export function ChatTemplateBehavior() {
  return (
    <div className={chatTemplateBehavior}>
      <p>How should Paddler obtain the chat template?</p>
      <div className={chatTemplateBehavior__optionList}>
        <label className={chatTemplateBehavior__option}>
          <div className={chatTemplateBehavior__radio}>
            <input
              type="radio"
              name="chat_template_behavior"
              required
              value="use_model_template"
            />
          </div>
          <div className={chatTemplateBehavior__description}>
            <p>
              <strong>
                Use the chat template provided by the model (recommended)
              </strong>
            </p>
            <p>
              Most models support this, but not all. If the model does not have
              a chat template, it will fail to load.
            </p>
          </div>
        </label>
        <label className={chatTemplateBehavior__option}>
          <div className={chatTemplateBehavior__radio}>
            <input
              type="radio"
              name="chat_template_behavior"
              required
              value="use_specific_template"
            />
          </div>
          <div className={chatTemplateBehavior__description}>
            <p>
              <strong>Use: </strong>
              <select>
                <optgroup label="Built-in template">
                  <option value="chatml">ChatML template</option>
                </optgroup>
              </select>
            </p>
            <p>
              Only use this option if you know the model does not come with a
              chat template, or you want to modify it to suit your needs.
            </p>
          </div>
        </label>
      </div>
    </div>
  );
}
