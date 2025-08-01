import clsx from "clsx";
import React from "react";
import { Link } from "wouter";

import { ChatTemplateHead } from "../schemas/ChatTemplateHead";

import iconNotes from "../../icons/notes.svg";
import {
  chatTemplatesPageTemplatesList,
  chatTemplatesPageTemplatesList__button,
  chatTemplatesPageTemplatesList__createTemplate,
  chatTemplatesPageTemplatesList__createTemplateActive,
} from "./ChatTemplatesPageTemplatesList.module.css";

function getActiveCreateLinkClassName(isActive: boolean) {
  return clsx({
    [chatTemplatesPageTemplatesList__button]: true,
    [chatTemplatesPageTemplatesList__createTemplate]: true,
    [chatTemplatesPageTemplatesList__createTemplateActive]: isActive,
  });
}

export function ChatTemplatesPageTemplatesList({
  chat_template_heads,
}: {
  chat_template_heads: ChatTemplateHead[];
}) {
  console.log("chat_template_heads", chat_template_heads);

  return (
    <div className={chatTemplatesPageTemplatesList}>
      <Link className={getActiveCreateLinkClassName} href="/chat-templates">
        <img src={iconNotes} alt="Create new chat template" />
        <span>New chat template</span>
      </Link>
    </div>
  );
}
