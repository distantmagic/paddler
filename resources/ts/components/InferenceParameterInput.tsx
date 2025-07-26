import React, { useCallback, useContext, type FormEvent } from "react";

import { InferenceParametersContext } from "../contexts/InferenceParametersContext";
import { type InferenceParameters } from "../InferenceParameters.type";
import {
  inferenceParameterInput,
  inferenceParameterInput__input,
  inferenceParameterInput__label,
} from "./InferenceParameterInput.module.css";

export function InferenceParameterInput({
  description,
  name,
}: {
  description: string;
  name: keyof InferenceParameters;
}) {
  const { parameters, setParameter } = useContext(InferenceParametersContext);

  const onInput = useCallback(
    function (event: FormEvent<HTMLInputElement>) {
      event.preventDefault();

      setParameter(name, parseFloat(event.currentTarget.value));
    },
    [name, setParameter],
  );

  return (
    <label className={inferenceParameterInput}>
      <abbr className={inferenceParameterInput__label} title={description}>
        {name}
      </abbr>
      <input
        className={inferenceParameterInput__input}
        onInput={onInput}
        required
        type="number"
        value={parameters[name]}
      />
    </label>
  );
}
