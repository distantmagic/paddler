import React, {
  useCallback,
  useContext,
  useEffect,
  useState,
  type MouseEvent,
} from "react";
import { useLocation } from "wouter";

import { ChatTemplateContext } from "../contexts/ChatTemplateContext";

import iconDelete from "../../icons/delete.svg";
import iconEdit from "../../icons/edit.svg";
import iconSave from "../../icons/save.svg";
import {
  chatTemplatesPageToolbar,
  chatTemplatesPageToolbar__button,
  chatTemplatesPageToolbar__track,
} from "./ChatTemplatesPageToolbar.module.css";

export function ChatTemplatesPageToolbar({
  managementAddr,
}: {
  managementAddr: string;
}) {
  const [wantsToSave, setWantsToSave] = useState(false);
  const [, navigate] = useLocation();
  const { content, exists, id, name, setName } =
    useContext(ChatTemplateContext);

  const onDeleteClick = useCallback(
    function (evt: MouseEvent<HTMLButtonElement>) {
      evt.preventDefault();

      if (!exists || !confirm("Are you sure?")) {
        return;
      }

      fetch(`//${managementAddr}/api/v1/chat_template/${id}`, {
        method: "DELETE",
      })
        .then(function (response) {
          if (response.ok) {
            navigate(`/`);
          } else {
            throw new Error(
              `Failed to delete a chat template: ${response.statusText}`,
            );
          }
        })
        .catch(function (error: unknown) {
          console.error("Error deleting chat template:", error);
        });
    },
    [exists, id, managementAddr, navigate],
  );

  const onRenameClick = useCallback(
    function (evt: MouseEvent<HTMLButtonElement>) {
      evt.preventDefault();

      const newName = prompt("Enter a new name for the template:", name);

      if (newName && newName !== name) {
        setName(newName);
        setWantsToSave(true);
      }
    },
    [name, setName, setWantsToSave],
  );

  const onSaveClick = useCallback(
    function (evt: MouseEvent<HTMLButtonElement>) {
      evt.preventDefault();

      setWantsToSave(true);
    },
    [setWantsToSave],
  );

  useEffect(
    function () {
      if (!wantsToSave) {
        return;
      }

      fetch(`//${managementAddr}/api/v1/chat_template`, {
        method: "PUT",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          content,
          id,
          name: exists ? name : prompt("Enter a name for the new template:"),
        }),
      })
        .then(function (response) {
          if (response.ok) {
            navigate(`/${id}`);
          } else {
            throw new Error(
              `Failed to save a chat template: ${response.statusText}`,
            );
          }
        })
        .catch(function (error: unknown) {
          console.error("Error saving chat template:", error);
        })
        .finally(function () {
          setWantsToSave(false);
        });
    },
    [
      content,
      exists,
      id,
      managementAddr,
      name,
      navigate,
      setWantsToSave,
      wantsToSave,
    ],
  );

  return (
    <div className={chatTemplatesPageToolbar}>
      <div className={chatTemplatesPageToolbar__track}>
        <button
          className={chatTemplatesPageToolbar__button}
          onClick={onSaveClick}
        >
          <img src={iconSave} alt="Save" />
          Save
        </button>
        {exists && (
          <button
            className={chatTemplatesPageToolbar__button}
            onClick={onRenameClick}
          >
            <img src={iconEdit} alt="Rename" />
            Rename
          </button>
        )}
      </div>
      <div className={chatTemplatesPageToolbar__track}>
        {exists && (
          <button
            className={chatTemplatesPageToolbar__button}
            onClick={onDeleteClick}
          >
            <img src={iconDelete} alt="Delete" />
            Delete
          </button>
        )}
      </div>
    </div>
  );
}
