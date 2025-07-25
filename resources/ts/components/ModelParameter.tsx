import React from "react";

import {
  modelParameter,
  modelParameter__head,
  modelParameter__input,
} from "./ModelParameter.module.css";

export function ModelParameter({
  defaultvalue,
  description,
  name,
}: {
  defaultvalue: number;
  description: string;
  name: string;
}) {
  return (
    <label className={modelParameter}>
      <div className={modelParameter__head}>
        <span>{description}</span>
        <abbr title={description}>{name}</abbr>
      </div>
      <input
        className={modelParameter__input}
        defaultValue={defaultvalue}
        required
        type="number"
      />
    </label>
  );
}
