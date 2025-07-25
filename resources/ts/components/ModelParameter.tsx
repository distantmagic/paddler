import React, { useCallback, useContext, type FormEvent } from "react";

import { ModelParametersContext } from "../contexts/ModelParametersContext";
import { type ModelParameters } from "../ModelParameters.type";
import {
  modelParameter,
  modelParameter__input,
  modelParameter__label,
} from "./ModelParameter.module.css";

export function ModelParameter({
  description,
  name,
}: {
  description: string;
  name: keyof ModelParameters;
}) {
  const { parameters, setParameter } = useContext(ModelParametersContext);

  const onInput = useCallback(
    function (event: FormEvent<HTMLInputElement>) {
      event.preventDefault();

      setParameter(name, parseFloat(event.currentTarget.value));
    },
    [name, setParameter],
  );

  return (
    <label className={modelParameter}>
      <abbr className={modelParameter__label} title={description}>
        {name}
      </abbr>
      <input
        className={modelParameter__input}
        onInput={onInput}
        required
        type="number"
        value={parameters[name]}
      />
    </label>
  );
}
