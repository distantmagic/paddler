import React, { useCallback, useContext, type FormEvent } from "react";

import { ModelParametersContext } from "../contexts/ModelParametersContext";
import { type ModelParameters } from "../ModelParameters.type";
import {
  modelParameterInput,
  modelParameterInput__input,
  modelParameterInput__label,
} from "./ModelParameterInput.module.css";

export function ModelParameterInput({
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
    <label className={modelParameterInput}>
      <abbr className={modelParameterInput__label} title={description}>
        {name}
      </abbr>
      <input
        className={modelParameterInput__input}
        onInput={onInput}
        required
        type="number"
        value={parameters[name]}
      />
    </label>
  );
}
