import clsx from "clsx";
import React from "react";

import { type StreamState } from "../hooks/useEventSourceUpdates";
import { ChatTemplateHeadsResponseSchema } from "../schemas/ChatTemplateHeadsResponse";

import {
  chatTemplateBehavior,
  chatTemplateBehavior__description,
  chatTemplateBehavior__option,
  chatTemplateBehavior__optionDisabled,
  chatTemplateBehavior__optionList,
  chatTemplateBehavior__radio,
} from "./ChatTemplateBehavior.module.css";

export function ChatTemplateBehavior({
  chatTemplateStreamUpdateState,
}: {
  chatTemplateStreamUpdateState: StreamState<
    typeof ChatTemplateHeadsResponseSchema
  >;
}) {
  const chatTemplateHeads =
    chatTemplateStreamUpdateState.data?.chat_template_heads;
  const chatTemplatesExist = chatTemplateHeads && chatTemplateHeads.length > 0;

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
        <label
          className={clsx(chatTemplateBehavior__option, {
            [chatTemplateBehavior__optionDisabled]: !chatTemplatesExist,
          })}
        >
          <div className={chatTemplateBehavior__radio}>
            <input
              disabled={!chatTemplatesExist}
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
                {chatTemplateStreamUpdateState.data?.chat_template_heads.map(
                  function (chat_template_head) {
                    return (
                      <option
                        key={chat_template_head.id}
                        value={chat_template_head.id}
                      >
                        {chat_template_head.name}
                      </option>
                    );
                  },
                )}
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
