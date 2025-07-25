import React from "react";

import {
  modelParameter,
  modelParameter__input,
  modelParameter__label,
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
      <abbr className={modelParameter__label} title={description}>
        {name}
      </abbr>
      <input
        className={modelParameter__input}
        defaultValue={defaultvalue}
        required
        type="number"
      />
    </label>
  );
}
