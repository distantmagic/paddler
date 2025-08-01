import React from "react";

import iconDelete from "../../icons/delete.svg";
import iconEdit from "../../icons/edit.svg";
import iconSave from "../../icons/save.svg";
import {
  chatTemplatesPageToolbar,
  chatTemplatesPageToolbar__button,
  chatTemplatesPageToolbar__track,
} from "./ChatTemplatesPageToolbar.module.css";

export function ChatTemplatesPageToolbar() {
  return (
    <div className={chatTemplatesPageToolbar}>
      <div className={chatTemplatesPageToolbar__track}>
        <button className={chatTemplatesPageToolbar__button}>
          <img src={iconSave} alt="Save" />
          Save
        </button>
        <button className={chatTemplatesPageToolbar__button}>
          <img src={iconEdit} alt="Rename" />
          Rename
        </button>
      </div>
      <div className={chatTemplatesPageToolbar__track}>
        <button className={chatTemplatesPageToolbar__button}>
          <img src={iconDelete} alt="Delete" />
          Delete
        </button>
      </div>
    </div>
  );
}
