import React from "react";

import {
  chatTemplateBehavior,
  chatTemplateBehavior__description,
  chatTemplateBehavior__option,
  chatTemplateBehavior__radio,
  chatTemplateBehavior__selectAlternative,
} from "./ChatTemplateBehavior.module.css";

export function ChatTemplateBehavior() {
  return (
    <div className={chatTemplateBehavior}>
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
          <div>Try to use chat template built into the model first.</div>
          <div>If there is no template embedded in the model, then:</div>
          <div className={chatTemplateBehavior__selectAlternative}>
            <select>
              <option value="panic">do not use any fallback template</option>
              <optgroup label="Your templates"></optgroup>
              <optgroup label="Built-in template">
                <option value="chatml">use ChatML template</option>
              </optgroup>
            </select>
          </div>
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
          <div>
            Always{" "}
            <select>
              <optgroup label="Your templates"></optgroup>
              <optgroup label="Built-in template">
                <option value="chatml">use ChatML template</option>
              </optgroup>
            </select>
          </div>
        </div>
      </label>
    </div>
  );
}
