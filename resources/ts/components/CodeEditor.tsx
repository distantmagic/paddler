import { jinja } from "@codemirror/lang-jinja";
import CodeMirror from "@uiw/react-codemirror";
import React from "react";

import { codeEditor } from "./CodeEditor.module.css";

export function CodeEditor({
  editable,
  onChange,
  value,
}:
  | {
      editable: true;
      value: string;
      onChange(this: void, value: string): void;
    }
  | {
      editable: false;
      value: string;
      onChange?: never;
    }) {
  return (
    <div className={codeEditor}>
      <CodeMirror
        editable={editable}
        extensions={[jinja()]}
        onChange={onChange}
        readOnly={!editable}
        value={value}
      />
    </div>
  );
}
