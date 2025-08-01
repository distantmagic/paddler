import clsx from "clsx";
import React from "react";
import { Link } from "wouter";

import { ChatTemplateHead } from "../schemas/ChatTemplateHead";

import iconAddNotes from "../../icons/add_notes.svg";
import iconNotes from "../../icons/notes.svg";
import {
  chatTemplatesPageTemplatesList,
  chatTemplatesPageTemplatesList__button,
  chatTemplatesPageTemplatesList__buttonActive,
} from "./ChatTemplatesPageTemplatesList.module.css";

function getActiveLinkClassName(isActive: boolean) {
  return clsx({
    [chatTemplatesPageTemplatesList__button]: true,
    [chatTemplatesPageTemplatesList__buttonActive]: isActive,
  });
}

export function ChatTemplatesPageTemplatesList({
  chat_template_heads,
}: {
  chat_template_heads: ChatTemplateHead[];
}) {
  return (
    <div className={chatTemplatesPageTemplatesList}>
      <Link className={getActiveLinkClassName} href="/">
        <img src={iconAddNotes} alt="Create new chat template" />
        <span>New chat template</span>
      </Link>
      {chat_template_heads.map(function ({ id, name }) {
        return (
          <Link key={id} className={getActiveLinkClassName} href={`/${id}`}>
            <img src={iconNotes} alt="Chat template" />
            <span>{name}</span>
          </Link>
        );
      })}
    </div>
  );
}
